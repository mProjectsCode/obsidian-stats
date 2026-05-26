use std::path::Path;

use crate::{
    alerts,
    constants::{
        DEFAULT_RELEASE_STATS_REFRESH_DAYS, RELEASE_CHANGELOG_PATH,
        RELEASE_GITHUB_INTERPOLATED_PATH, RELEASE_GITHUB_RAW_PATH, RELEASE_STATS_STATE_PATH,
    },
    file_utils::{read_chunked_data_or_default, write_in_chunks_atomic},
    github::RateLimitMode,
    state::{is_fresh, now_unix_seconds, read_json_or_default, write_json_atomic},
};
use data_lib::release::GithubReleaseInfo;
use serde::{Deserialize, Serialize};

mod github_fetch;
mod github_transform;
mod interpolate;
mod obsidian_feed;

use github_fetch::{fetch_github_release_entries, refresh_completed};
use github_transform::get_github_release_info;
use interpolate::interpolate_github_release_info;
use obsidian_feed::get_obs_release_info;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ReleaseStatsState {
    last_fetch_unix: Option<i64>,
    latest_etag: Option<String>,
}

pub fn build_release_stats(force: bool) -> Result<(), Box<dyn std::error::Error>> {
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

    let should_refresh = force
        || raw_github_info.is_empty()
        || !state
            .last_fetch_unix
            .is_some_and(|last| is_fresh(last, refresh_days));

    println!(
        "Release stats: refresh window={} days, force={}, should_refresh={}",
        refresh_days, force, should_refresh
    );

    if should_refresh {
        let fetch_outcome =
            fetch_github_release_entries(&rate_limit_mode, state.latest_etag.as_deref());
        let refresh_was_completed = refresh_completed(&fetch_outcome);
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
        if fetch_outcome.hit_unexpected_error {
            eprintln!(
                "Warning: unexpected error while refreshing release stats. Saved partial update."
            );
        }

        println!(
            "Release stats fetch summary: pages={}, entries_received={}, not_modified={}",
            fetch_outcome.page_count, new_entry_count, fetch_outcome.not_modified
        );

        if refresh_was_completed {
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

    let release_changelog = get_obs_release_info().inspect_err(|error| {
        alerts::record_unexpected_error("release changelog fetch", error.to_string());
    })?;

    write_in_chunks_atomic(Path::new(RELEASE_CHANGELOG_PATH), &release_changelog, 50)?;

    println!("Changelog data: {:#?}", time2.elapsed());

    println!("Release stats built in {:#?}", time.elapsed());

    Ok(())
}
