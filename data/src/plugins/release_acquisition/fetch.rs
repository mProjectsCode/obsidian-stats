use reqwest::blocking::{Client, Response};
use serde::Deserialize;

use crate::{alerts, github::RateLimitMode, state::now_unix_seconds};

use super::{
    PluginReleaseStateEntry, ReleaseFetchStatus,
    cache::{MainJsCacheOutcome, MainJsDownloadError, save_main_js_to_cache},
};

#[derive(Debug, Clone, Deserialize)]
struct GithubReleaseAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GithubRelease {
    tag_name: String,
    published_at: String,
    assets: Vec<GithubReleaseAsset>,
}

pub(super) enum ReleaseFetchResult {
    NotModified,
    Updated(Box<PluginReleaseStateEntry>, Option<MainJsCacheOutcome>),
}

pub(super) struct ReleaseFetchRequest<'a> {
    pub plugin_id: &'a str,
    pub repo: &'a str,
    pub target_release_tag: &'a str,
    pub previous_entry: Option<&'a PluginReleaseStateEntry>,
    pub previous_etag: Option<&'a str>,
}

enum ResponseHandling {
    Retry,
    Done(ReleaseFetchResult),
    Success {
        release: GithubRelease,
        response_etag: Option<String>,
    },
}

pub(super) fn fetch_release_info(
    request: ReleaseFetchRequest<'_>,
    client: &Client,
    rate_limit_mode: &RateLimitMode,
) -> ReleaseFetchResult {
    let mut retries = 0;

    loop {
        let response = match send_release_metadata_request(&request, client) {
            Ok(response) => response,
            Err(result) => return result,
        };

        match handle_release_metadata_response(&request, response, rate_limit_mode, &mut retries) {
            ResponseHandling::Retry => continue,
            ResponseHandling::Done(result) => return result,
            ResponseHandling::Success {
                release,
                response_etag,
            } => {
                return build_success_result(&request, client, release, response_etag);
            }
        }
    }
}

fn send_release_metadata_request(
    request: &ReleaseFetchRequest<'_>,
    client: &Client,
) -> Result<Response, ReleaseFetchResult> {
    let encoded_tag = encode_github_release_tag_for_path(request.target_release_tag);
    let mut http_request = client
        .get(format!(
            "https://api.github.com/repos/{}/releases/tags/{encoded_tag}",
            request.repo
        ))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "obsidian-stats-data");

    if let Some(etag) = request.previous_etag {
        http_request = http_request.header("If-None-Match", etag);
    }

    if let Ok(token) = std::env::var("GITHUB_TOKEN")
        && !token.is_empty()
    {
        http_request = http_request.bearer_auth(token);
    }

    http_request.send().map_err(|err| {
        alerts::record_unexpected_error(
            format!("plugin release metadata fetch for {}", request.plugin_id),
            err.to_string(),
        );
        ReleaseFetchResult::Updated(
            Box::new(state_entry(
                request.repo,
                None,
                None,
                ReleaseFetchStatus::RequestError(err.to_string()),
            )),
            None,
        )
    })
}

fn handle_release_metadata_response(
    request: &ReleaseFetchRequest<'_>,
    response: Response,
    rate_limit_mode: &RateLimitMode,
    retries: &mut usize,
) -> ResponseHandling {
    let status = response.status();
    let response_etag = response_etag(&response);

    if rate_limit_remaining_is_zero(response.headers()) {
        let detail = github_error_detail(status.as_u16(), response);
        alerts::record_rate_limit(
            format!("plugin release metadata fetch for {}", request.plugin_id),
            detail,
        );

        return ResponseHandling::Done(ReleaseFetchResult::Updated(
            Box::new(state_entry(
                request.repo,
                response_etag,
                None,
                ReleaseFetchStatus::RateLimited,
            )),
            None,
        ));
    }

    if status.as_u16() == 304 {
        return ResponseHandling::Done(ReleaseFetchResult::NotModified);
    }

    if status.as_u16() == 404 {
        return ResponseHandling::Done(ReleaseFetchResult::Updated(
            Box::new(state_entry(
                request.repo,
                response_etag,
                None,
                ReleaseFetchStatus::NoReleaseForVersion,
            )),
            None,
        ));
    }

    if status.as_u16() == 403 || status.as_u16() == 429 {
        let retry_wait = retry_wait_seconds(response.headers());

        if matches!(rate_limit_mode, RateLimitMode::Sleep)
            && *retries < 1
            && let Some(wait_secs) = retry_wait
            && wait_secs > 0
        {
            std::thread::sleep(std::time::Duration::from_secs(wait_secs as u64));
            *retries += 1;
            return ResponseHandling::Retry;
        }

        let detail = github_error_detail(status.as_u16(), response);
        alerts::record_rate_limit(
            format!("plugin release metadata fetch for {}", request.plugin_id),
            detail,
        );

        return ResponseHandling::Done(ReleaseFetchResult::Updated(
            Box::new(state_entry(
                request.repo,
                response_etag,
                None,
                ReleaseFetchStatus::RateLimited,
            )),
            None,
        ));
    }

    if !status.is_success() {
        let detail = github_error_detail(status.as_u16(), response);
        alerts::record_unexpected_error(
            format!("plugin release metadata fetch for {}", request.plugin_id),
            detail,
        );
        return ResponseHandling::Done(ReleaseFetchResult::Updated(
            Box::new(state_entry(
                request.repo,
                response_etag,
                None,
                ReleaseFetchStatus::HttpError(status.as_u16().to_string()),
            )),
            None,
        ));
    }

    match response.json::<GithubRelease>() {
        Ok(release) => ResponseHandling::Success {
            release,
            response_etag,
        },
        Err(err) => {
            alerts::record_unexpected_error(
                format!("plugin release metadata parse for {}", request.plugin_id),
                err.to_string(),
            );
            ResponseHandling::Done(ReleaseFetchResult::Updated(
                Box::new(state_entry(
                    request.repo,
                    response_etag,
                    None,
                    ReleaseFetchStatus::ParseError(err.to_string()),
                )),
                None,
            ))
        }
    }
}

fn build_success_result(
    request: &ReleaseFetchRequest<'_>,
    client: &Client,
    release: GithubRelease,
    response_etag: Option<String>,
) -> ReleaseFetchResult {
    if release.tag_name != request.target_release_tag {
        alerts::record_unexpected_error(
            format!("plugin release metadata fetch for {}", request.plugin_id),
            format!(
                "GitHub returned release tag {}, expected {}",
                release.tag_name, request.target_release_tag
            ),
        );
        return ReleaseFetchResult::Updated(
            Box::new(state_entry(
                request.repo,
                response_etag,
                None,
                ReleaseFetchStatus::ReleaseTagMismatch,
            )),
            None,
        );
    }

    let main_js_asset = release.assets.iter().find(|asset| asset.name == "main.js");
    let size = main_js_asset.map(|asset| asset.size);
    let (status, cache_outcome, successful_tag, successful_published_at) =
        handle_main_js_asset(request, client, &release, main_js_asset);

    ReleaseFetchResult::Updated(
        Box::new(PluginReleaseStateEntry {
            repo: request.repo.to_string(),
            last_checked_unix: now_unix_seconds(),
            latest_release_etag: response_etag,
            latest_release_main_js_size_bytes: size,
            last_successful_main_js_release_tag: successful_tag,
            last_successful_main_js_release_published_at: successful_published_at,
            estimated_target_es_version: None,
            latest_release_tag: Some(release.tag_name),
            latest_release_published_at: Some(release.published_at),
            latest_release_fetch_status: Some(status.as_state_value()),
        }),
        cache_outcome,
    )
}

fn handle_main_js_asset(
    request: &ReleaseFetchRequest<'_>,
    client: &Client,
    release: &GithubRelease,
    main_js_asset: Option<&GithubReleaseAsset>,
) -> (
    ReleaseFetchStatus,
    Option<MainJsCacheOutcome>,
    Option<String>,
    Option<String>,
) {
    let Some(asset) = main_js_asset else {
        return (ReleaseFetchStatus::NoMainJsAsset, None, None, None);
    };

    if !should_download_main_js_for_release(
        request.previous_entry,
        &release.tag_name,
        &release.published_at,
    ) {
        return (
            ReleaseFetchStatus::MainJsNotUpdatedSinceSuccess,
            None,
            None,
            None,
        );
    }

    match save_main_js_to_cache(
        client,
        request.plugin_id,
        &release.tag_name,
        &asset.browser_download_url,
        asset.size,
    ) {
        Ok(outcome) => (
            ReleaseFetchStatus::Ok,
            Some(outcome),
            Some(release.tag_name.clone()),
            Some(release.published_at.clone()),
        ),
        Err(err) => {
            record_main_js_download_error(request.plugin_id, &err);
            (err.status(), None, None, None)
        }
    }
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

fn record_main_js_download_error(plugin_id: &str, error: &MainJsDownloadError) {
    match error {
        MainJsDownloadError::RateLimited(_) => {
            alerts::record_rate_limit(
                format!("plugin release main.js download for {plugin_id}"),
                error.detail_message(),
            );
        }
        _ => alerts::record_unexpected_error(
            format!("plugin release main.js download for {plugin_id}"),
            error.detail_message(),
        ),
    }
}

fn state_entry(
    repo: &str,
    etag: Option<String>,
    release_tag: Option<String>,
    status: ReleaseFetchStatus,
) -> PluginReleaseStateEntry {
    PluginReleaseStateEntry {
        repo: repo.to_string(),
        last_checked_unix: now_unix_seconds(),
        latest_release_etag: etag,
        latest_release_main_js_size_bytes: None,
        last_successful_main_js_release_tag: None,
        last_successful_main_js_release_published_at: None,
        estimated_target_es_version: None,
        latest_release_tag: release_tag,
        latest_release_published_at: None,
        latest_release_fetch_status: Some(status.as_state_value()),
    }
}

fn response_etag(response: &Response) -> Option<String> {
    response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
}

fn retry_wait_seconds(headers: &reqwest::header::HeaderMap) -> Option<i64> {
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

fn rate_limit_remaining_is_zero(headers: &reqwest::header::HeaderMap) -> bool {
    headers
        .get("x-ratelimit-remaining")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|value| value.trim() == "0")
}

fn github_error_detail(status_code: u16, response: Response) -> String {
    let headers = github_diagnostic_headers(response.headers());
    let body = response
        .text()
        .ok()
        .map(|body| trim_diagnostic_body(&body))
        .filter(|body| !body.is_empty());

    let mut detail = format!("GitHub returned HTTP {status_code}");
    if !headers.is_empty() {
        detail.push_str("; headers: ");
        detail.push_str(&headers);
    }
    if let Some(body) = body {
        detail.push_str("; body: ");
        detail.push_str(&body);
    }

    detail
}

fn github_diagnostic_headers(headers: &reqwest::header::HeaderMap) -> String {
    const HEADER_NAMES: &[&str] = &[
        "retry-after",
        "x-ratelimit-limit",
        "x-ratelimit-remaining",
        "x-ratelimit-reset",
        "x-ratelimit-resource",
        "x-ratelimit-used",
        "x-github-request-id",
        "x-oauth-scopes",
        "x-accepted-oauth-scopes",
    ];

    HEADER_NAMES
        .iter()
        .filter_map(|name| {
            headers
                .get(*name)
                .and_then(|value| value.to_str().ok())
                .map(|value| format!("{name}={value}"))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn trim_diagnostic_body(body: &str) -> String {
    const MAX_BODY_CHARS: usize = 1000;

    let body = body.trim();
    let char_count = body.chars().count();
    if char_count <= MAX_BODY_CHARS {
        return body.to_string();
    }

    let mut trimmed = body.chars().take(MAX_BODY_CHARS).collect::<String>();
    trimmed.push_str("...");
    trimmed
}

fn encode_github_release_tag_for_path(tag: &str) -> String {
    let mut encoded = String::new();

    for byte in tag.as_bytes() {
        let ch = *byte as char;
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '~') {
            encoded.push(ch);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use super::{
        encode_github_release_tag_for_path, github_diagnostic_headers,
        rate_limit_remaining_is_zero, should_download_main_js_for_release, trim_diagnostic_body,
    };
    use crate::plugins::release_acquisition::PluginReleaseStateEntry;
    use reqwest::header::{HeaderMap, HeaderValue};

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

    #[test]
    fn encodes_release_tag_as_url_path_segment() {
        assert_eq!(encode_github_release_tag_for_path("1.2.3"), "1.2.3");
        assert_eq!(encode_github_release_tag_for_path("beta/1"), "beta%2F1");
        assert_eq!(encode_github_release_tag_for_path("v 1.0.0"), "v%201.0.0");
    }

    #[test]
    fn formats_github_diagnostic_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-ratelimit-remaining", HeaderValue::from_static("0"));
        headers.insert("x-ratelimit-resource", HeaderValue::from_static("core"));
        headers.insert("x-github-request-id", HeaderValue::from_static("abc:123"));
        headers.insert("content-type", HeaderValue::from_static("application/json"));

        assert_eq!(
            github_diagnostic_headers(&headers),
            "x-ratelimit-remaining=0, x-ratelimit-resource=core, x-github-request-id=abc:123"
        );
    }

    #[test]
    fn trims_long_diagnostic_bodies() {
        let body = "a".repeat(1001);
        let trimmed = trim_diagnostic_body(&body);

        assert_eq!(trimmed.chars().count(), 1003);
        assert!(trimmed.ends_with("..."));
    }

    #[test]
    fn detects_zero_rate_limit_remaining_header() {
        let mut headers = HeaderMap::new();
        headers.insert("x-ratelimit-remaining", HeaderValue::from_static("0"));

        assert!(rate_limit_remaining_is_zero(&headers));
    }
}
