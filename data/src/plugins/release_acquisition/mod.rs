use std::{
    path::Path,
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
    constants::{DEFAULT_PLUGIN_RELEASE_REFRESH_DAYS, PLUGIN_RELEASE_ENRICHMENT_STATE_PATH},
    github::RateLimitMode,
    plugins::stats_helper::{HelperPluginStore, TargetReleaseError},
    progress::should_log_progress,
    security::http_client,
    state::{is_fresh, now_unix_seconds, read_json_or_default, write_json_atomic},
};

mod cache;
mod fetch;

use cache::MainJsCacheOutcome;
pub use cache::release_main_js_cache_path;
use fetch::{ReleaseFetchRequest, ReleaseFetchResult, fetch_release_info};

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

#[derive(Debug, Clone, PartialEq, Eq)]
enum ReleaseFetchStatus {
    Ok,
    NotModified,
    VersionHistoryMissing,
    TargetRelease(TargetReleaseError),
    NoReleaseForVersion,
    NoMainJsAsset,
    MainJsNotUpdatedSinceSuccess,
    MainJsRateLimited,
    RateLimited,
    ReleaseTagMismatch,
    RequestError(String),
    HttpError(String),
    ParseError(String),
    MainJsDownloadFailed(String),
    Unknown(String),
}

impl ReleaseFetchStatus {
    fn from_state_value(status: &str) -> Self {
        if let Some(detail) = status.strip_prefix("request_error:") {
            return Self::RequestError(detail.to_string());
        }
        if let Some(detail) = status.strip_prefix("http_error:") {
            return Self::HttpError(detail.to_string());
        }
        if let Some(detail) = status.strip_prefix("parse_error:") {
            return Self::ParseError(detail.to_string());
        }
        if let Some(detail) = status.strip_prefix("main_js_download_failed:") {
            return Self::MainJsDownloadFailed(detail.to_string());
        }

        match status {
            "ok" => Self::Ok,
            "not_modified" => Self::NotModified,
            "version_history_missing" => Self::VersionHistoryMissing,
            "helper_plugin_missing" => Self::TargetRelease(TargetReleaseError::HelperPluginMissing),
            "manifest_missing" => Self::TargetRelease(TargetReleaseError::ManifestMissing),
            "manifest_version_missing" => {
                Self::TargetRelease(TargetReleaseError::ManifestVersionMissing)
            }
            "manifest_version_invalid" => {
                Self::TargetRelease(TargetReleaseError::ManifestVersionInvalid)
            }
            "manifest_version_prefixed" => {
                Self::TargetRelease(TargetReleaseError::ManifestVersionPrefixed)
            }
            "release_for_manifest_version_missing" => {
                Self::TargetRelease(TargetReleaseError::ReleaseForManifestVersionMissing)
            }
            "no_release_for_version" => Self::NoReleaseForVersion,
            "no_main_js_asset" => Self::NoMainJsAsset,
            "main_js_not_updated_since_success" => Self::MainJsNotUpdatedSinceSuccess,
            "main_js_rate_limited" => Self::MainJsRateLimited,
            "main_js_download_failed" => Self::MainJsDownloadFailed(String::new()),
            "rate_limited" => Self::RateLimited,
            "release_tag_mismatch" => Self::ReleaseTagMismatch,
            _ => Self::Unknown(status.to_string()),
        }
    }

    fn as_state_value(&self) -> String {
        match self {
            Self::Ok => "ok".to_string(),
            Self::NotModified => "not_modified".to_string(),
            Self::VersionHistoryMissing => "version_history_missing".to_string(),
            Self::TargetRelease(error) => error.as_state_value().to_string(),
            Self::NoReleaseForVersion => "no_release_for_version".to_string(),
            Self::NoMainJsAsset => "no_main_js_asset".to_string(),
            Self::MainJsNotUpdatedSinceSuccess => "main_js_not_updated_since_success".to_string(),
            Self::MainJsRateLimited => "main_js_rate_limited".to_string(),
            Self::RateLimited => "rate_limited".to_string(),
            Self::ReleaseTagMismatch => "release_tag_mismatch".to_string(),
            Self::RequestError(detail) => format!("request_error:{detail}"),
            Self::HttpError(detail) => format!("http_error:{detail}"),
            Self::ParseError(detail) => format!("parse_error:{detail}"),
            Self::MainJsDownloadFailed(detail) if detail.is_empty() => {
                "main_js_download_failed".to_string()
            }
            Self::MainJsDownloadFailed(detail) => format!("main_js_download_failed:{detail}"),
            Self::Unknown(status) => status.clone(),
        }
    }

    fn is_retryable_main_js(&self) -> bool {
        matches!(
            self,
            Self::MainJsRateLimited | Self::MainJsDownloadFailed(_)
        )
    }

    fn is_retryable_release_fetch(&self) -> bool {
        matches!(
            self,
            Self::RateLimited | Self::RequestError(_) | Self::HttpError(_) | Self::ParseError(_)
        ) || self.is_retryable_main_js()
    }
}

fn should_retry_release_fetch(entry: &PluginReleaseStateEntry) -> bool {
    entry
        .latest_release_fetch_status
        .as_deref()
        .map(ReleaseFetchStatus::from_state_value)
        .is_some_and(|status| status.is_retryable_release_fetch())
}

struct ReleaseAcquireJob {
    key: String,
    plugin_id: String,
    repo: String,
    target_release_tag: String,
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
    force: bool,
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
        "Release acquisition: processing {} plugins (refresh window: {} days, force: {}, threads: {})",
        plugins.len(),
        refresh_days,
        force,
        thread_count
    );

    let mut state: PluginReleaseState =
        read_json_or_default(Path::new(PLUGIN_RELEASE_ENRICHMENT_STATE_PATH));
    let mut stats = AcquireRunStats::default();
    let helper_store = HelperPluginStore::read()?;

    let mut jobs = Vec::new();

    for plugin in plugins {
        if plugin.removed_commit.is_some() {
            stats.skipped_removed += 1;
            continue;
        }

        let key = plugin.id.clone();
        let repo = plugin.current_entry.repo.clone();

        let previous_entry = previous_entry_for_repo(&state, &key, &repo).cloned();
        let target_release_tag = match helper_store.target_release_for_plugin(plugin) {
            Ok(target) => target.tag,
            Err(error) => {
                let entry = target_release_error_state_entry(&repo, previous_entry.as_ref(), error);
                if let Some(status) = &entry.latest_release_fetch_status {
                    *stats.status_counts.entry(status.clone()).or_insert(0) += 1;
                }
                state.entries.insert(key, entry);
                continue;
            }
        };

        if let Some(entry) = &previous_entry
            && entry.repo == repo
            && entry.latest_release_tag.as_deref() == Some(target_release_tag.as_str())
            && !should_retry_release_fetch(entry)
            && !force
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
            target_release_tag,
            previous_entry,
        });
    }

    if !jobs.is_empty() {
        stats.fetched_http += jobs.len();
        let total_jobs = jobs.len();

        let client = http_client()?;
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

fn target_release_error_state_entry(
    repo: &str,
    previous_entry: Option<&PluginReleaseStateEntry>,
    error: TargetReleaseError,
) -> PluginReleaseStateEntry {
    PluginReleaseStateEntry {
        repo: repo.to_string(),
        last_checked_unix: now_unix_seconds(),
        latest_release_etag: None,
        latest_release_main_js_size_bytes: None,
        last_successful_main_js_release_tag: previous_entry
            .and_then(|prev| prev.last_successful_main_js_release_tag.clone()),
        last_successful_main_js_release_published_at: previous_entry
            .and_then(|prev| prev.last_successful_main_js_release_published_at.clone()),
        estimated_target_es_version: previous_entry
            .and_then(|prev| prev.estimated_target_es_version.clone()),
        latest_release_tag: None,
        latest_release_published_at: None,
        latest_release_fetch_status: Some(
            ReleaseFetchStatus::TargetRelease(error).as_state_value(),
        ),
    }
}

fn previous_entry_for_repo<'a>(
    state: &'a PluginReleaseState,
    plugin_id: &str,
    repo: &str,
) -> Option<&'a PluginReleaseStateEntry> {
    state
        .entries
        .get(plugin_id)
        .filter(|entry| entry.repo == repo)
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
        .filter(|entry| {
            entry.latest_release_tag.as_deref() == Some(job.target_release_tag.as_str())
        })
        .and_then(|entry| entry.latest_release_etag.as_deref());

    let (mut entry, cache_outcome, not_modified) = match fetch_release_info(
        ReleaseFetchRequest {
            plugin_id: &job.plugin_id,
            repo: &job.repo,
            target_release_tag: &job.target_release_tag,
            previous_entry: job.previous_entry.as_ref(),
            previous_etag,
        },
        client,
        rate_limit_mode,
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
                latest_release_fetch_status: Some(ReleaseFetchStatus::NotModified.as_state_value()),
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

fn configured_thread_count(env_var: &str, default_threads: usize) -> usize {
    std::env::var(env_var)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|count| *count > 0)
        .unwrap_or(default_threads)
}

#[cfg(test)]
mod tests {
    use super::{
        PluginReleaseState, PluginReleaseStateEntry, ReleaseFetchStatus, previous_entry_for_repo,
        should_retry_release_fetch,
    };
    use hashbrown::HashMap;

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
    fn previous_entry_for_repo_ignores_stale_repo_state() {
        let state = PluginReleaseState {
            entries: HashMap::from([("plugin".to_string(), state_entry("ok"))]),
        };

        assert!(previous_entry_for_repo(&state, "plugin", "owner/repo").is_some());
        assert!(previous_entry_for_repo(&state, "plugin", "new-owner/repo").is_none());
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
        assert!(!should_retry_release_fetch(&state_entry(
            "no_release_for_version"
        )));
        assert!(!should_retry_release_fetch(&state_entry(
            "version_history_missing"
        )));
        assert!(!should_retry_release_fetch(&state_entry(
            "manifest_version_prefixed"
        )));
    }

    #[test]
    fn release_fetch_status_round_trips_known_payload_statuses() {
        for status in [
            "request_error:timeout",
            "http_error:502",
            "parse_error:broken json",
            "main_js_download_failed:write:disk full",
        ] {
            assert_eq!(
                ReleaseFetchStatus::from_state_value(status).as_state_value(),
                status
            );
        }
    }
}
