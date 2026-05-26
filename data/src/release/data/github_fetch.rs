use reqwest::header::HeaderMap;

use crate::{
    alerts, constants::RELEASE_STATS_URL, github::RateLimitMode, release::GithubReleaseEntry,
    security::http_client, state::now_unix_seconds,
};

pub(super) struct FetchOutcome {
    pub(super) entries: Vec<GithubReleaseEntry>,
    pub(super) hit_rate_limit: bool,
    pub(super) hit_unexpected_error: bool,
    pub(super) not_modified: bool,
    pub(super) page_count: usize,
    pub(super) latest_etag: Option<String>,
}

pub(super) fn refresh_completed(fetch_outcome: &FetchOutcome) -> bool {
    fetch_outcome.not_modified
        || (!fetch_outcome.hit_rate_limit && !fetch_outcome.hit_unexpected_error)
}

pub(super) fn fetch_github_release_entries(
    rate_limit_mode: &RateLimitMode,
    previous_etag: Option<&str>,
) -> FetchOutcome {
    let mut current_link = Some(RELEASE_STATS_URL.to_string());
    let mut release_entries: Vec<GithubReleaseEntry> = vec![];
    let mut hit_rate_limit = false;
    let mut hit_unexpected_error = false;
    let mut not_modified = false;
    let mut page_count = 0;
    let mut latest_etag = None;
    let client = match http_client() {
        Ok(client) => client,
        Err(error) => {
            alerts::record_unexpected_error("release stats HTTP client", error.to_string());
            return FetchOutcome {
                entries: release_entries,
                hit_rate_limit,
                hit_unexpected_error: true,
                not_modified,
                page_count,
                latest_etag,
            };
        }
    };
    let mut first_request = true;

    while let Some(api_link) = current_link.clone() {
        let mut request = client
            .get(api_link)
            .header("Accept", "application/json")
            .header("User-Agent", "obsidian-stats-data");

        if first_request && let Some(etag) = previous_etag {
            request = request.header("If-None-Match", etag);
        }

        if let Ok(token) = std::env::var("GITHUB_TOKEN")
            && !token.is_empty()
        {
            request = request.bearer_auth(token);
        }

        let response = match request.send() {
            Ok(response) => response,
            Err(error) => {
                hit_unexpected_error = true;
                alerts::record_unexpected_error("release stats fetch", error.to_string());
                eprintln!("Failed to fetch release stats: {error}");
                break;
            }
        };

        if first_request {
            latest_etag = response
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string());
        }

        if response.status().as_u16() == 403 || response.status().as_u16() == 429 {
            hit_rate_limit = true;
            alerts::record_rate_limit(
                "release stats fetch",
                format!("GitHub returned HTTP {}", response.status().as_u16()),
            );

            if matches!(rate_limit_mode, RateLimitMode::Sleep)
                && let Some(wait) = extract_retry_wait_seconds(response.headers())
                && wait > 0
            {
                std::thread::sleep(std::time::Duration::from_secs(wait as u64));
                continue;
            }

            eprintln!("Rate limited while fetching release stats.");
            break;
        }

        if response.status().as_u16() == 304 {
            not_modified = true;
            break;
        }

        if response.status().is_success() {
            page_count += 1;
            current_link = extract_next_link(response.headers());
            println!("  Release stats page {page_count} fetched");

            let json: Vec<GithubReleaseEntry> = match response.json() {
                Ok(json) => json,
                Err(error) => {
                    hit_unexpected_error = true;
                    alerts::record_unexpected_error("release stats parse", error.to_string());
                    eprintln!("Failed to parse release stats JSON: {error}");
                    break;
                }
            };
            release_entries.extend(json);
            first_request = false;
        } else {
            hit_unexpected_error = true;
            alerts::record_unexpected_error(
                "release stats fetch",
                format!("GitHub returned HTTP {}", response.status().as_u16()),
            );
            eprintln!("Failed to fetch release stats: {}", response.status());
            break;
        }
    }

    FetchOutcome {
        entries: release_entries,
        hit_rate_limit,
        hit_unexpected_error,
        not_modified,
        page_count,
        latest_etag,
    }
}

fn extract_next_link(headers: &HeaderMap) -> Option<String> {
    // <https://api.github.com/repositories/262342594/releases?page=2>; rel="next", <https://api.github.com/repositories/262342594/releases?page=6>; rel="last"
    headers.get(reqwest::header::LINK).and_then(|link_header| {
        link_header.to_str().ok().and_then(|link_str| {
            link_str
                .split(',')
                .find_map(|s| {
                    if s.contains("rel=\"next\"") {
                        s.split(';')
                            .next()
                            .map(|s| s.trim().trim_matches('<').trim_matches('>'))
                    } else {
                        None
                    }
                })
                .map(String::from)
        })
    })
}

fn extract_retry_wait_seconds(headers: &HeaderMap) -> Option<i64> {
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
    use super::{FetchOutcome, refresh_completed};

    fn outcome(
        hit_rate_limit: bool,
        hit_unexpected_error: bool,
        not_modified: bool,
    ) -> FetchOutcome {
        FetchOutcome {
            entries: Vec::new(),
            hit_rate_limit,
            hit_unexpected_error,
            not_modified,
            page_count: 0,
            latest_etag: None,
        }
    }

    #[test]
    fn unexpected_errors_do_not_count_as_completed_refresh() {
        assert!(!refresh_completed(&outcome(false, true, false)));
        assert!(!refresh_completed(&outcome(true, false, false)));
        assert!(refresh_completed(&outcome(false, false, false)));
        assert!(refresh_completed(&outcome(false, true, true)));
    }
}
