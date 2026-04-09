use std::path::Path;

use crate::{
    constants::{
        DEFAULT_RELEASE_STATS_REFRESH_DAYS, GITHUB_RATE_LIMIT_MODE_ENV, RELEASE_CHANGELOG_PATH,
        RELEASE_GITHUB_INTERPOLATED_PATH, RELEASE_GITHUB_RAW_PATH, RELEASE_INFO_URL,
        RELEASE_STATS_STATE_PATH, RELEASE_STATS_URL,
    },
    file_utils::{read_chunked_data_or_default, write_in_chunks_atomic},
    release::GithubReleaseEntry,
    state::{is_fresh, now_unix_seconds, read_json_or_default, write_json_atomic},
};
use data_lib::{
    date::Date,
    input_data::ObsReleasesFeedInner,
    release::{GithubAssetInfo, GithubReleaseInfo, ObsidianPlatform, ObsidianReleaseInfo},
    version::Version,
};
use hashbrown::HashMap;
use reqwest::{blocking::Client, header::HeaderMap};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ReleaseStatsState {
    last_fetch_unix: Option<i64>,
    latest_etag: Option<String>,
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

struct FetchOutcome {
    entries: Vec<GithubReleaseEntry>,
    hit_rate_limit: bool,
    not_modified: bool,
    page_count: usize,
    latest_etag: Option<String>,
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

fn fetch_github_release_entries(
    rate_limit_mode: &RateLimitMode,
    previous_etag: Option<&str>,
) -> FetchOutcome {
    let mut current_link = Some(RELEASE_STATS_URL.to_string());
    let mut release_entries: Vec<GithubReleaseEntry> = vec![];
    let mut hit_rate_limit = false;
    let mut not_modified = false;
    let mut page_count = 0;
    let mut latest_etag = None;
    let client = Client::new();
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

            let json: Vec<GithubReleaseEntry> =
                response.json().expect("Failed to parse release stats JSON");
            release_entries.extend(json);
            first_request = false;
        } else {
            eprintln!("Failed to fetch release stats: {}", response.status());
            break;
        }
    }

    FetchOutcome {
        entries: release_entries,
        hit_rate_limit,
        not_modified,
        page_count,
        latest_etag,
    }
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

fn get_github_release_info(
    release_entries: &mut Vec<GithubReleaseInfo>,
    new_entries: Vec<GithubReleaseEntry>,
) {
    let today = Date::now();

    new_entries.into_iter().for_each(|entry| {
        let version = Version::parse(&entry.tag_name).expect("Failed to parse version");

        let existing_entry = release_entries.iter_mut().find(|e| e.version == version);

        if let Some(existing_entry) = existing_entry {
            entry.assets.iter().for_each(|asset| {
                let existing_asset = existing_entry
                    .assets
                    .iter_mut()
                    .find(|a| a.name == asset.name);
                if let Some(existing_asset) = existing_asset {
                    existing_asset
                        .downloads
                        .insert(today.to_fancy_string(), asset.download_count);
                } else {
                    let mut download_map = HashMap::new();
                    download_map.insert(today.to_fancy_string(), asset.download_count);

                    existing_entry.assets.push(GithubAssetInfo {
                        name: asset.name.clone(),
                        downloads: download_map,
                        size: asset.size,
                    });
                }
            });
        } else {
            release_entries.push(GithubReleaseInfo {
                version: version.clone(),
                date: Date::from_string(&entry.published_at.date_naive().to_string())
                    .expect("Failed to parse date"),
                time: entry.published_at.time().to_string(),
                assets: entry
                    .assets
                    .into_iter()
                    .map(|asset| {
                        let mut download_map = HashMap::new();
                        download_map.insert(today.to_fancy_string(), asset.download_count);

                        GithubAssetInfo {
                            name: asset.name,
                            downloads: download_map,
                            size: asset.size,
                        }
                    })
                    .collect(),
            });
        }
    });
}

fn interpolate(a: u32, b: u32, ratio: f64) -> u32 {
    if ratio >= 1.0 {
        b
    } else if ratio <= 0.0 {
        a
    } else {
        (a as f64 * (1.0 - ratio) + b as f64 * ratio) as u32
    }
}

fn interpolate_github_release_info(full_data: &[GithubReleaseInfo]) -> Vec<GithubReleaseInfo> {
    let today = Date::now();

    full_data
        .iter()
        .map(|info| {
            let assets = info
                .assets
                .iter()
                .map(|asset| {
                    if asset.downloads.is_empty() {
                        return asset.clone();
                    }

                    let mut updates = asset
                        .downloads
                        .keys()
                        .map(|x| Date::from_string(x).expect("valid date"))
                        .collect::<Vec<_>>();
                    updates.sort();

                    let mut first_update = updates
                        .first()
                        .expect("Expected at least one download entry")
                        .clone();

                    first_update.reverse_days(7);
                    first_update.advance_to_weekday(0);

                    let downloads = first_update
                        .iterate_weekly_to(&today)
                        .filter_map(|date| {
                            let (pre, post) = date.find_surrounding(&updates)?;
                            let pre_diff = date.diff_in_days(pre).abs();
                            let post_diff = date.diff_in_days(post).abs();
                            let ratio = if pre_diff + post_diff == 0 {
                                0.0
                            } else {
                                pre_diff as f64 / (pre_diff + post_diff) as f64
                            };

                            let pre_downloads =
                                asset.downloads.get(&pre.to_fancy_string()).unwrap();
                            let post_downloads =
                                asset.downloads.get(&post.to_fancy_string()).unwrap();

                            let downloads = interpolate(*pre_downloads, *post_downloads, ratio);

                            Some((date.to_fancy_string(), downloads))
                        })
                        .collect::<HashMap<String, u32>>();

                    GithubAssetInfo {
                        name: asset.name.clone(),
                        downloads,
                        size: asset.size,
                    }
                })
                .collect();

            GithubReleaseInfo {
                version: info.version.clone(),
                date: info.date.clone(),
                time: info.time.clone(),
                assets,
            }
        })
        .collect()
}

fn get_obs_release_info() -> Vec<ObsidianReleaseInfo> {
    let response = reqwest::blocking::get(RELEASE_INFO_URL).expect("Failed to fetch release info");
    let text = response.text().expect("Failed to read response text");

    // dbg!(&text);

    let feed_data: ObsReleasesFeedInner =
        quick_xml::de::from_str(&text).expect("Failed to parse release feed");

    feed_data
        .entries
        .into_iter()
        .filter_map(|entry| {
            if entry.id.contains("publish") {
                return None; // Skip publish entries, because they are weird and we don't use them
            }

            let id_parts = entry.id.split('-').collect::<Vec<&str>>();
            let version_str = id_parts.last()?.trim_matches('/');
            let version = Version::parse(version_str)?;

            let insider = entry.title.contains("Early access");

            // Check that the title contains a version of the form "X.Y" or "X.Y.Z"
            // This assumes that the title contains no other dots
            let major_release = entry.title.split('.').count() == 2;

            let platform = if entry.id.contains("desktop") {
                ObsidianPlatform::Desktop
            } else if entry.id.contains("mobile") {
                ObsidianPlatform::Mobile
            } else if entry.id.contains("publish") {
                ObsidianPlatform::Publish
            } else {
                ObsidianPlatform::Desktop // Default to Desktop if not specified
            };

            let date = Date::from_string(entry.updated.split('T').next()?)?;

            Some(ObsidianReleaseInfo {
                version,
                platform,
                insider,
                date,
                info: entry.content,
                major_release,
            })
        })
        .collect()
}

pub fn build_release_stats() -> Result<(), Box<dyn std::error::Error>> {
    let time = std::time::Instant::now();
    let mut time2 = std::time::Instant::now();

    let refresh_days = std::env::var("RELEASE_STATS_REFRESH_DAYS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(DEFAULT_RELEASE_STATS_REFRESH_DAYS);
    let rate_limit_mode = RateLimitMode::from_env();
    let mut state: ReleaseStatsState = read_json_or_default(Path::new(RELEASE_STATS_STATE_PATH));

    let mut raw_github_info: Vec<GithubReleaseInfo> =
        read_chunked_data_or_default(Path::new(RELEASE_GITHUB_RAW_PATH));

    let should_refresh = raw_github_info.is_empty()
        || !state
            .last_fetch_unix
            .is_some_and(|last| is_fresh(last, refresh_days));

    println!(
        "Release stats: refresh window={} days, should_refresh={}",
        refresh_days, should_refresh
    );

    if should_refresh {
        let fetch_outcome =
            fetch_github_release_entries(&rate_limit_mode, state.latest_etag.as_deref());
        let new_entry_count = fetch_outcome.entries.len();

        if fetch_outcome.not_modified {
            println!("Release stats: GitHub returned 304 Not Modified.");
        } else {
            get_github_release_info(&mut raw_github_info, fetch_outcome.entries);
        }

        if fetch_outcome.hit_rate_limit {
            eprintln!(
                "Warning: hit GitHub rate limit while refreshing release stats. Saved partial update."
            );
        }

        println!(
            "Release stats fetch summary: pages={}, entries_received={}, not_modified={}",
            fetch_outcome.page_count, new_entry_count, fetch_outcome.not_modified
        );

        let refresh_completed = fetch_outcome.not_modified || !fetch_outcome.hit_rate_limit;
        if refresh_completed {
            state.last_fetch_unix = Some(now_unix_seconds());
        } else {
            eprintln!(
                "Release stats refresh did not complete, so freshness timestamp was not advanced."
            );
        }

        if fetch_outcome.latest_etag.is_some() {
            state.latest_etag = fetch_outcome.latest_etag;
        }
        write_json_atomic(Path::new(RELEASE_STATS_STATE_PATH), &state)?;
    } else {
        println!("Skipping release stats refresh because data is still fresh.");
    }

    let interpolated_github_info = interpolate_github_release_info(&raw_github_info);

    write_in_chunks_atomic(Path::new(RELEASE_GITHUB_RAW_PATH), &raw_github_info, 50)?;
    write_in_chunks_atomic(
        Path::new(RELEASE_GITHUB_INTERPOLATED_PATH),
        &interpolated_github_info,
        50,
    )?;

    println!("Github release data: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    let release_changelog = get_obs_release_info();

    write_in_chunks_atomic(Path::new(RELEASE_CHANGELOG_PATH), &release_changelog, 50)?;

    println!("Changelog data: {:#?}", time2.elapsed());

    println!("Release stats built in {:#?}", time.elapsed());

    Ok(())
}
