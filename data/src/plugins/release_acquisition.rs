use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use data_lib::plugin::PluginData;
use hashbrown::HashMap;
use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelIterator, ParallelIterator},
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::{
    alerts,
    constants::{
        DEFAULT_PLUGIN_RELEASE_REFRESH_DAYS, GITHUB_RATE_LIMIT_MODE_ENV,
        PLUGIN_RELEASE_ENRICHMENT_STATE_PATH, PLUGIN_RELEASE_MAIN_JS_PATH,
    },
    progress::should_log_progress,
    state::{is_fresh, now_unix_seconds, read_json_or_default, write_json_atomic},
};

const MAX_MAIN_JS_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024; // 512 MB
const PLUGIN_RELEASE_THREADS_ENV: &str = "PLUGIN_RELEASE_THREADS";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginReleaseState {
    pub entries: HashMap<String, PluginReleaseStateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReleaseStateEntry {
    pub repo: String,
    pub last_checked_unix: i64,
    pub latest_release_etag: Option<String>,
    pub latest_release_main_js_size_bytes: Option<u64>,
    pub last_successful_main_js_release_tag: Option<String>,
    pub last_successful_main_js_release_published_at: Option<String>,
    pub estimated_target_es_version: Option<String>,
    pub latest_release_tag: Option<String>,
    pub latest_release_published_at: Option<String>,
    pub latest_release_fetch_status: Option<String>,
}

#[derive(Debug, Clone)]
enum RateLimitMode {
    Defer,
    Sleep,
}

impl RateLimitMode {
    fn from_env() -> Self {
        match std::env::var(GITHUB_RATE_LIMIT_MODE_ENV)
            .unwrap_or_else(|_| "defer".to_string())
            .to_lowercase()
            .as_str()
        {
            "sleep" => Self::Sleep,
            _ => Self::Defer,
        }
    }
}

fn should_retry_main_js_download(entry: &PluginReleaseStateEntry) -> bool {
    entry
        .latest_release_fetch_status
        .as_deref()
        .is_some_and(|status| {
            status == "main_js_rate_limited"
                || status == "main_js_download_failed"
                || status.starts_with("main_js_download_failed:")
        })
}

fn should_retry_release_fetch(entry: &PluginReleaseStateEntry) -> bool {
    entry
        .latest_release_fetch_status
        .as_deref()
        .is_some_and(|status| {
            status == "rate_limited"
                || should_retry_main_js_download(entry)
                || status.starts_with("request_error:")
                || status.starts_with("http_error:")
                || status.starts_with("parse_error:")
        })
}

fn should_download_main_js_for_release(
    previous_entry: Option<&PluginReleaseStateEntry>,
    release_tag: &str,
    release_published_at: &str,
) -> bool {
    let Some(previous_entry) = previous_entry else {
        return true;
    };

    let Some(last_success_published_at) = previous_entry
        .last_successful_main_js_release_published_at
        .as_deref()
    else {
        return true;
    };

    if release_published_at > last_success_published_at {
        return true;
    }

    if release_published_at < last_success_published_at {
        return false;
    }

    previous_entry
        .last_successful_main_js_release_tag
        .as_deref()
        .is_none_or(|last_success_tag| last_success_tag != release_tag)
}

#[derive(Debug, Clone, Deserialize)]
struct GithubLatestReleaseAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GithubLatestRelease {
    tag_name: String,
    published_at: String,
    assets: Vec<GithubLatestReleaseAsset>,
}

enum ReleaseFetchResult {
    NotModified,
    Updated(Box<PluginReleaseStateEntry>, Option<MainJsCacheOutcome>),
}

#[derive(Clone, Copy)]
enum MainJsCacheOutcome {
    Downloaded,
    Reused,
}

struct ReleaseAcquireJob {
    key: String,
    plugin_id: String,
    repo: String,
    previous_entry: Option<PluginReleaseStateEntry>,
}

struct ReleaseAcquireJobResult {
    key: String,
    entry: PluginReleaseStateEntry,
    cache_outcome: Option<MainJsCacheOutcome>,
    not_modified: bool,
}

#[derive(Default)]
struct AcquireRunStats {
    skipped_removed: usize,
    skipped_fresh: usize,
    fetched_http: usize,
    not_modified: usize,
    downloaded_main_js: usize,
    reused_main_js: usize,
    status_counts: HashMap<String, usize>,
}

pub fn acquire_plugin_release_main_js(
    plugins: &[PluginData],
) -> Result<(), Box<dyn std::error::Error>> {
    let refresh_days = std::env::var("PLUGIN_RELEASE_REFRESH_DAYS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(DEFAULT_PLUGIN_RELEASE_REFRESH_DAYS);
    let rate_limit_mode = RateLimitMode::from_env();
    let default_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let thread_count = configured_thread_count(PLUGIN_RELEASE_THREADS_ENV, default_threads);

    println!(
        "Release acquisition: processing {} plugins (refresh window: {} days, threads: {})",
        plugins.len(),
        refresh_days,
        thread_count
    );

    let mut state: PluginReleaseState =
        read_json_or_default(Path::new(PLUGIN_RELEASE_ENRICHMENT_STATE_PATH));
    let mut stats = AcquireRunStats::default();

    let mut jobs = Vec::new();

    for plugin in plugins {
        if plugin.removed_commit.is_some() {
            stats.skipped_removed += 1;
            continue;
        }

        let key = plugin.id.clone();
        let repo = plugin.current_entry.repo.clone();

        let previous_entry = state.entries.get(&key).cloned();
        if let Some(entry) = &previous_entry
            && entry.repo == repo
            && !should_retry_release_fetch(entry)
            && is_fresh(entry.last_checked_unix, refresh_days)
        {
            stats.skipped_fresh += 1;
            if let Some(status) = &entry.latest_release_fetch_status {
                *stats.status_counts.entry(status.clone()).or_insert(0) += 1;
            }
            continue;
        }

        jobs.push(ReleaseAcquireJob {
            key,
            plugin_id: plugin.id.clone(),
            repo,
            previous_entry,
        });
    }

    if !jobs.is_empty() {
        stats.fetched_http += jobs.len();
        let total_jobs = jobs.len();

        let client = Client::new();
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build()
            .expect("Failed to build release acquisition thread pool");
        let processed = AtomicUsize::new(0);

        let results = thread_pool.install(|| {
            jobs.into_par_iter()
                .map(|job| {
                    let result = process_release_job(job, &client, &rate_limit_mode);
                    let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
                    if should_log_progress(done, total_jobs) {
                        println!("  Release acquisition progress: {done} / {total_jobs}");
                    }
                    result
                })
                .collect::<Vec<_>>()
        });

        for result in results {
            if result.not_modified {
                stats.not_modified += 1;
            }

            if let Some(cache_outcome) = result.cache_outcome {
                match cache_outcome {
                    MainJsCacheOutcome::Downloaded => stats.downloaded_main_js += 1,
                    MainJsCacheOutcome::Reused => stats.reused_main_js += 1,
                }
            }

            if let Some(status) = &result.entry.latest_release_fetch_status {
                *stats.status_counts.entry(status.clone()).or_insert(0) += 1;
            }

            state.entries.insert(result.key, result.entry);
        }
    }

    write_json_atomic(Path::new(PLUGIN_RELEASE_ENRICHMENT_STATE_PATH), &state)?;

    println!("Release acquisition summary:");
    println!("  Skipped (removed): {}", stats.skipped_removed);
    println!("  Skipped (fresh): {}", stats.skipped_fresh);
    println!("  HTTP checks: {}", stats.fetched_http);
    println!("  Not modified (ETag): {}", stats.not_modified);
    println!("  main.js downloaded: {}", stats.downloaded_main_js);
    println!("  main.js reused: {}", stats.reused_main_js);

    let mut status_counts = stats.status_counts.into_iter().collect::<Vec<_>>();
    status_counts.sort_by(|a, b| a.0.cmp(&b.0));
    for (status, count) in status_counts {
        println!("  Release status {status}: {count}");
    }

    Ok(())
}

fn process_release_job(
    job: ReleaseAcquireJob,
    client: &Client,
    rate_limit_mode: &RateLimitMode,
) -> ReleaseAcquireJobResult {
    let previous_entry = job.previous_entry.clone();

    let previous_etag = job
        .previous_entry
        .as_ref()
        .filter(|entry| !should_retry_release_fetch(entry))
        .and_then(|entry| entry.latest_release_etag.as_deref());

    let (mut entry, cache_outcome, not_modified) = match fetch_latest_release_info(
        &job.plugin_id,
        &job.repo,
        job.previous_entry.as_ref(),
        client,
        rate_limit_mode,
        previous_etag,
    ) {
        ReleaseFetchResult::NotModified => {
            let mut reused = previous_entry.clone().unwrap_or(PluginReleaseStateEntry {
                repo: job.repo.clone(),
                last_checked_unix: now_unix_seconds(),
                latest_release_etag: None,
                latest_release_main_js_size_bytes: None,
                last_successful_main_js_release_tag: None,
                last_successful_main_js_release_published_at: None,
                estimated_target_es_version: None,
                latest_release_tag: None,
                latest_release_published_at: None,
                latest_release_fetch_status: Some("not_modified".to_string()),
            });
            reused.last_checked_unix = now_unix_seconds();
            (reused, None, true)
        }
        ReleaseFetchResult::Updated(entry, cache_outcome) => (*entry, cache_outcome, false),
    };

    entry.repo = job.repo;
    entry.last_checked_unix = now_unix_seconds();

    if entry.last_successful_main_js_release_tag.is_none() {
        entry.last_successful_main_js_release_tag = previous_entry
            .as_ref()
            .and_then(|prev| prev.last_successful_main_js_release_tag.clone());
    }
    if entry.last_successful_main_js_release_published_at.is_none() {
        entry.last_successful_main_js_release_published_at = previous_entry
            .as_ref()
            .and_then(|prev| prev.last_successful_main_js_release_published_at.clone());
    }

    ReleaseAcquireJobResult {
        key: job.key,
        entry,
        cache_outcome,
        not_modified,
    }
}

fn fetch_latest_release_info(
    plugin_id: &str,
    repo: &str,
    previous_entry: Option<&PluginReleaseStateEntry>,
    client: &Client,
    rate_limit_mode: &RateLimitMode,
    previous_etag: Option<&str>,
) -> ReleaseFetchResult {
    let mut retries = 0;

    loop {
        let mut request = client
            .get(format!(
                "https://api.github.com/repos/{repo}/releases/latest"
            ))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "obsidian-stats-data");

        if let Some(etag) = previous_etag {
            request = request.header("If-None-Match", etag);
        }

        if let Ok(token) = std::env::var("GITHUB_TOKEN")
            && !token.is_empty()
        {
            request = request.bearer_auth(token);
        }

        let response = match request.send() {
            Ok(response) => response,
            Err(err) => {
                alerts::record_unexpected_error(
                    format!("plugin release metadata fetch for {plugin_id}"),
                    err.to_string(),
                );
                return ReleaseFetchResult::Updated(
                    Box::new(PluginReleaseStateEntry {
                        repo: repo.to_string(),
                        last_checked_unix: now_unix_seconds(),
                        latest_release_etag: None,
                        latest_release_main_js_size_bytes: None,
                        last_successful_main_js_release_tag: None,
                        last_successful_main_js_release_published_at: None,
                        estimated_target_es_version: None,
                        latest_release_tag: None,
                        latest_release_published_at: None,
                        latest_release_fetch_status: Some(format!("request_error:{err}")),
                    }),
                    None,
                );
            }
        };

        let status = response.status();

        if status.as_u16() == 304 {
            return ReleaseFetchResult::NotModified;
        }

        let response_etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        if status.as_u16() == 404 {
            return ReleaseFetchResult::Updated(
                Box::new(PluginReleaseStateEntry {
                    repo: repo.to_string(),
                    last_checked_unix: now_unix_seconds(),
                    latest_release_etag: response_etag,
                    latest_release_main_js_size_bytes: None,
                    last_successful_main_js_release_tag: None,
                    last_successful_main_js_release_published_at: None,
                    estimated_target_es_version: None,
                    latest_release_tag: None,
                    latest_release_published_at: None,
                    latest_release_fetch_status: Some("no_release".to_string()),
                }),
                None,
            );
        }

        if status.as_u16() == 403 || status.as_u16() == 429 {
            alerts::record_rate_limit(
                format!("plugin release metadata fetch for {plugin_id}"),
                format!("GitHub returned HTTP {}", status.as_u16()),
            );
            if matches!(rate_limit_mode, RateLimitMode::Sleep)
                && retries < 1
                && let Some(wait_secs) = extract_retry_wait_seconds(response.headers())
                && wait_secs > 0
            {
                std::thread::sleep(std::time::Duration::from_secs(wait_secs as u64));
                retries += 1;
                continue;
            }

            return ReleaseFetchResult::Updated(
                Box::new(PluginReleaseStateEntry {
                    repo: repo.to_string(),
                    last_checked_unix: now_unix_seconds(),
                    latest_release_etag: response_etag,
                    latest_release_main_js_size_bytes: None,
                    last_successful_main_js_release_tag: None,
                    last_successful_main_js_release_published_at: None,
                    estimated_target_es_version: None,
                    latest_release_tag: None,
                    latest_release_published_at: None,
                    latest_release_fetch_status: Some("rate_limited".to_string()),
                }),
                None,
            );
        }

        if !status.is_success() {
            alerts::record_unexpected_error(
                format!("plugin release metadata fetch for {plugin_id}"),
                format!("GitHub returned HTTP {}", status.as_u16()),
            );
            return ReleaseFetchResult::Updated(
                Box::new(PluginReleaseStateEntry {
                    repo: repo.to_string(),
                    last_checked_unix: now_unix_seconds(),
                    latest_release_etag: response_etag,
                    latest_release_main_js_size_bytes: None,
                    last_successful_main_js_release_tag: None,
                    last_successful_main_js_release_published_at: None,
                    estimated_target_es_version: None,
                    latest_release_tag: None,
                    latest_release_published_at: None,
                    latest_release_fetch_status: Some(format!("http_error:{}", status.as_u16())),
                }),
                None,
            );
        }

        let parsed: Result<GithubLatestRelease, _> = response.json();
        match parsed {
            Ok(latest) => {
                let main_js_asset = latest.assets.iter().find(|asset| asset.name == "main.js");
                let size = main_js_asset.map(|asset| asset.size);
                let (status, cache_outcome, successful_tag, successful_published_at) = if let Some(
                    asset,
                ) =
                    main_js_asset
                {
                    if should_download_main_js_for_release(
                        previous_entry,
                        &latest.tag_name,
                        &latest.published_at,
                    ) {
                        match save_main_js_to_cache(
                            client,
                            plugin_id,
                            &latest.tag_name,
                            &asset.browser_download_url,
                            asset.size,
                        ) {
                            Ok(outcome) => (
                                "ok".to_string(),
                                Some(outcome),
                                Some(latest.tag_name.clone()),
                                Some(latest.published_at.clone()),
                            ),
                            Err(err) => {
                                match &err {
                                    MainJsDownloadError::RateLimited(_) => {
                                        alerts::record_rate_limit(
                                            format!(
                                                "plugin release main.js download for {plugin_id}"
                                            ),
                                            err.detail_message(),
                                        );
                                    }
                                    _ => alerts::record_unexpected_error(
                                        format!("plugin release main.js download for {plugin_id}"),
                                        err.detail_message(),
                                    ),
                                }

                                (err.status_label(), None, None, None)
                            }
                        }
                    } else {
                        (
                            "main_js_not_updated_since_success".to_string(),
                            None,
                            None,
                            None,
                        )
                    }
                } else {
                    ("no_main_js_asset".to_string(), None, None, None)
                };

                return ReleaseFetchResult::Updated(
                    Box::new(PluginReleaseStateEntry {
                        repo: repo.to_string(),
                        last_checked_unix: now_unix_seconds(),
                        latest_release_etag: response_etag,
                        latest_release_main_js_size_bytes: size,
                        last_successful_main_js_release_tag: successful_tag,
                        last_successful_main_js_release_published_at: successful_published_at,
                        estimated_target_es_version: None,
                        latest_release_tag: Some(latest.tag_name),
                        latest_release_published_at: Some(latest.published_at),
                        latest_release_fetch_status: Some(status),
                    }),
                    cache_outcome,
                );
            }
            Err(err) => {
                alerts::record_unexpected_error(
                    format!("plugin release metadata parse for {plugin_id}"),
                    err.to_string(),
                );
                return ReleaseFetchResult::Updated(
                    Box::new(PluginReleaseStateEntry {
                        repo: repo.to_string(),
                        last_checked_unix: now_unix_seconds(),
                        latest_release_etag: response_etag,
                        latest_release_main_js_size_bytes: None,
                        last_successful_main_js_release_tag: None,
                        last_successful_main_js_release_published_at: None,
                        estimated_target_es_version: None,
                        latest_release_tag: None,
                        latest_release_published_at: None,
                        latest_release_fetch_status: Some(format!("parse_error:{err}")),
                    }),
                    None,
                );
            }
        }
    }
}

enum MainJsDownloadError {
    RateLimited(u16),
    InvalidSize(u64),
    Request(String),
    Http(u16),
    Read(String),
    SizeMismatch { expected: u64, actual: u64 },
    Write(String),
}

impl MainJsDownloadError {
    fn status_label(&self) -> String {
        match self {
            Self::RateLimited(_) => "main_js_rate_limited".to_string(),
            Self::InvalidSize(size) => format!("main_js_download_failed:invalid_size:{size}"),
            Self::Request(err) => format!("main_js_download_failed:request:{err}"),
            Self::Http(status) => format!("main_js_download_failed:http:{status}"),
            Self::Read(err) => format!("main_js_download_failed:read:{err}"),
            Self::SizeMismatch { expected, actual } => {
                format!("main_js_download_failed:size_mismatch:{expected}:{actual}")
            }
            Self::Write(err) => format!("main_js_download_failed:write:{err}"),
        }
    }

    fn detail_message(&self) -> String {
        match self {
            Self::RateLimited(status) => format!("GitHub returned HTTP {status}"),
            Self::InvalidSize(size) => format!("asset size {size} bytes is outside allowed bounds"),
            Self::Request(err) => err.clone(),
            Self::Http(status) => format!("GitHub returned HTTP {status}"),
            Self::Read(err) => err.clone(),
            Self::SizeMismatch { expected, actual } => {
                format!("expected {expected} bytes, downloaded {actual} bytes")
            }
            Self::Write(err) => err.clone(),
        }
    }
}

fn save_main_js_to_cache(
    client: &Client,
    plugin_id: &str,
    release_tag: &str,
    download_url: &str,
    size: u64,
) -> Result<MainJsCacheOutcome, MainJsDownloadError> {
    if size == 0 || size > MAX_MAIN_JS_DOWNLOAD_BYTES {
        return Err(MainJsDownloadError::InvalidSize(size));
    }

    let cache_path = release_main_js_cache_path(plugin_id, release_tag);
    if let Ok(meta) = fs::metadata(&cache_path)
        && meta.len() == size
    {
        return Ok(MainJsCacheOutcome::Reused);
    }

    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut request = client
        .get(download_url)
        .header("Accept", "application/octet-stream")
        .header("User-Agent", "obsidian-stats-data");

    if let Ok(token) = std::env::var("GITHUB_TOKEN")
        && !token.is_empty()
    {
        request = request.bearer_auth(token);
    }

    let response = match request.send() {
        Ok(resp) => resp,
        Err(err) => return Err(MainJsDownloadError::Request(err.to_string())),
    };

    if response.status().as_u16() == 403 || response.status().as_u16() == 429 {
        return Err(MainJsDownloadError::RateLimited(response.status().as_u16()));
    }

    if !response.status().is_success() {
        return Err(MainJsDownloadError::Http(response.status().as_u16()));
    }

    let bytes = match response.bytes() {
        Ok(bytes) => bytes,
        Err(err) => return Err(MainJsDownloadError::Read(err.to_string())),
    };

    if bytes.len() as u64 != size {
        return Err(MainJsDownloadError::SizeMismatch {
            expected: size,
            actual: bytes.len() as u64,
        });
    }

    let write_result = File::create(cache_path).and_then(|file| {
        let mut writer = BufWriter::new(file);
        writer.write_all(bytes.as_ref())?;
        writer.flush()
    });

    match write_result {
        Ok(()) => Ok(MainJsCacheOutcome::Downloaded),
        Err(error) => Err(MainJsDownloadError::Write(error.to_string())),
    }
}

fn configured_thread_count(env_var: &str, default_threads: usize) -> usize {
    std::env::var(env_var)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|count| *count > 0)
        .unwrap_or(default_threads)
}

pub fn release_main_js_cache_path(plugin_id: &str, release_tag: &str) -> PathBuf {
    let sanitized_tag = release_tag
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();

    Path::new(PLUGIN_RELEASE_MAIN_JS_PATH)
        .join(plugin_id)
        .join(format!("{sanitized_tag}-main.js"))
}

fn extract_retry_wait_seconds(headers: &reqwest::header::HeaderMap) -> Option<i64> {
    let retry_after = headers
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok());
    if retry_after.is_some() {
        return retry_after;
    }

    let reset_unix = headers
        .get("x-ratelimit-reset")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())?;

    let now = now_unix_seconds();
    Some((reset_unix - now).max(0))
}

#[cfg(test)]
mod tests {
    use super::{
        PluginReleaseStateEntry, should_download_main_js_for_release, should_retry_release_fetch,
    };

    fn state_entry(status: &str) -> PluginReleaseStateEntry {
        PluginReleaseStateEntry {
            repo: "owner/repo".to_string(),
            last_checked_unix: 0,
            latest_release_etag: None,
            latest_release_main_js_size_bytes: None,
            last_successful_main_js_release_tag: None,
            last_successful_main_js_release_published_at: None,
            estimated_target_es_version: None,
            latest_release_tag: None,
            latest_release_published_at: None,
            latest_release_fetch_status: Some(status.to_string()),
        }
    }

    #[test]
    fn retries_transient_release_fetch_errors() {
        assert!(should_retry_release_fetch(&state_entry("rate_limited")));
        assert!(should_retry_release_fetch(&state_entry(
            "request_error:timeout"
        )));
        assert!(should_retry_release_fetch(&state_entry("http_error:500")));
        assert!(should_retry_release_fetch(&state_entry(
            "parse_error:broken json"
        )));
        assert!(should_retry_release_fetch(&state_entry(
            "main_js_rate_limited"
        )));
        assert!(should_retry_release_fetch(&state_entry(
            "main_js_download_failed:write:disk full"
        )));
        assert!(!should_retry_release_fetch(&state_entry("ok")));
        assert!(!should_retry_release_fetch(&state_entry("no_release")));
    }

    #[test]
    fn downloads_main_js_when_release_is_newer_than_last_success() {
        let mut entry = state_entry("ok");
        entry.last_successful_main_js_release_tag = Some("1.0.0".to_string());
        entry.last_successful_main_js_release_published_at =
            Some("2024-01-01T00:00:00Z".to_string());

        assert!(should_download_main_js_for_release(
            Some(&entry),
            "1.1.0",
            "2024-02-01T00:00:00Z"
        ));
        assert!(!should_download_main_js_for_release(
            Some(&entry),
            "1.1.0",
            "2023-12-01T00:00:00Z"
        ));
    }
}
