use data_lib::{
    date::Date,
    input_data::{ObsDownloadStats, ObsPluginList},
    plugin::PluginData,
};
use hashbrown::{HashMap, HashSet};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    path::Path,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    constants::{OBS_RELEASES_REPO_PATH, PLUGIN_DATA_PATH, PLUGIN_LIST_PATH, PLUGIN_STATS_PATH},
    file_utils::{read_chunked_data, write_in_chunks_atomic},
    git_utils::get_obs_repo_changes,
    plugins::{BorrowedPluginData, PluginDownloadStats, PluginList},
    progress::should_log_progress,
};

fn load_plugin_list_history() -> Vec<PluginList> {
    let commits = get_obs_repo_changes();
    let total_commits = commits.len();

    assert!(!commits.is_empty(), "No plugin list changes found");

    println!("Loading plugin list history from {total_commits} commits...");
    let processed = AtomicUsize::new(0);

    commits
        .par_iter()
        .filter_map(|commit| {
            let list = Command::new("git")
                .args([
                    "cat-file",
                    "-p",
                    &format!("{}:{}", commit.hash, PLUGIN_LIST_PATH),
                ])
                .current_dir(
                    Path::new(OBS_RELEASES_REPO_PATH)
                        .canonicalize()
                        .expect("Failed to canonicalize path"),
                )
                .output()
                .expect("Failed to execute git command");

            let list_str = String::from_utf8_lossy(&list.stdout).to_string();
            if list_str.is_empty() {
                eprintln!("Empty plugin list at commit {}", commit.to_fancy_string());
                return None;
            }
            let parsed_list: Result<ObsPluginList, serde_json::Error> =
                serde_json::from_str(&list_str);
            let result = match parsed_list {
                Ok(list) => Some(PluginList {
                    entries: list.to_hashmap(),
                    commit: commit.clone(),
                }),
                Err(e) => {
                    eprintln!(
                        "Error parsing plugin list at commit {}: {}",
                        commit.to_fancy_string(),
                        e
                    );
                    None
                }
            };

            let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
            if should_log_progress(done, total_commits) {
                println!("  Plugin list history progress: {done} / {total_commits}");
            }

            result
        })
        .collect()
}

fn build_plugin_change_timeline(plugin_lists: &[PluginList]) -> Vec<BorrowedPluginData<'_>> {
    println!("Building plugin data...");

    let mut plugin_data_map = HashMap::new();

    assert!(!plugin_lists.is_empty(), "No plugin lists found");

    for (id, entry) in &plugin_lists[0].entries {
        plugin_data_map.insert(
            id.clone(),
            BorrowedPluginData::new(id.clone(), &plugin_lists[0].commit, entry),
        );
    }

    let total_lists = plugin_lists.len();
    for (idx, plugin_list) in plugin_lists.iter().enumerate().skip(1) {
        for (_, plugin) in plugin_data_map.iter_mut() {
            plugin.find_changes(plugin_list);
        }

        for (id, entry) in &plugin_list.entries {
            if !plugin_data_map.contains_key(id) {
                plugin_data_map.insert(
                    id.clone(),
                    BorrowedPluginData::new(id.clone(), &plugin_list.commit, entry),
                );
            }
        }

        if should_log_progress(idx + 1, total_lists) {
            println!("  Plugin timeline progress: {} / {}", idx + 1, total_lists);
        }
    }

    plugin_data_map.into_iter().map(|(_, data)| data).collect()
}

fn backfill_download_history(
    plugin_data: &mut [BorrowedPluginData],
    download_stats: &[PluginDownloadStats],
) {
    println!("Updating weekly download stats...");

    let mut download_stats_map = HashMap::new();
    for stat in download_stats {
        download_stats_map.insert(stat.get_date(), stat);
    }

    let start_date = Date::new(2020, 1, 1);
    let end_date = Date::now();
    let total_days = start_date.diff_in_days(&end_date).unsigned_abs() as usize + 1;
    let mut processed_days = 0;

    // Something in May 2024 is broken in source data (for example advanced-canvas).
    let excluded_dates = [
        Date::new(2024, 5, 18),
        Date::new(2024, 5, 19),
        Date::new(2024, 5, 20),
        Date::new(2024, 5, 21),
        Date::new(2024, 5, 22),
        Date::new(2024, 5, 23),
        Date::new(2024, 5, 24),
        Date::new(2024, 5, 25),
        Date::new(2024, 5, 26),
        Date::new(2024, 5, 27),
        Date::new(2024, 5, 28),
    ]
    .into_iter()
    .collect::<HashSet<_>>();

    for date in start_date.iterate_daily_to(&end_date) {
        processed_days += 1;
        if excluded_dates.contains(&date) {
            if should_log_progress(processed_days, total_days) {
                println!(
                    "  Download backfill progress: {} / {} days",
                    processed_days, total_days
                );
            }
            continue;
        }

        let Some(stats) = find_recent_download_stats(&download_stats_map, &date) else {
            if should_log_progress(processed_days, total_days) {
                println!(
                    "  Download backfill progress: {} / {} days",
                    processed_days, total_days
                );
            }
            continue;
        };

        for entry in plugin_data.iter_mut() {
            // Don't update downloads for plugins that were not yet released
            if entry.added_commit.date > date {
                continue;
            }

            entry.update_download_history(stats);
        }

        if should_log_progress(processed_days, total_days) {
            println!(
                "  Download backfill progress: {} / {} days",
                processed_days, total_days
            );
        }
    }
}

fn find_recent_download_stats<'a>(
    download_stats_map: &'a HashMap<Date, &'a PluginDownloadStats>,
    date: &Date,
) -> Option<&'a PluginDownloadStats> {
    for i in 0..6 {
        let mut current_date = date.clone();
        current_date.advance_days(i);

        if let Some(stats) = download_stats_map.get(&current_date) {
            return Some(stats);
        }
    }

    None
}

fn build_version_history(
    plugin_data: &mut [BorrowedPluginData],
    download_stats: &[PluginDownloadStats],
) {
    println!("Updating version history...");

    let total_stats = download_stats.len();
    for (idx, stat) in download_stats.iter().enumerate() {
        for entry in plugin_data.iter_mut() {
            entry.update_version_history(stat);
        }

        if should_log_progress(idx + 1, total_stats) {
            println!(
                "  Version history progress: {} / {} snapshots",
                idx + 1,
                total_stats
            );
        }
    }

    for entry in plugin_data.iter_mut() {
        entry.sort_version_history();
    }
}

fn load_plugin_download_stat_history() -> Vec<PluginDownloadStats> {
    println!("Fetching plugin download stats...");

    let commits = get_obs_repo_changes();
    let total_commits = commits.len();
    println!("Loading plugin download stats from {total_commits} commits...");
    let processed = AtomicUsize::new(0);

    commits
        .par_iter()
        .filter_map(|commit| {
            let stats = Command::new("git")
                .args([
                    "cat-file",
                    "-p",
                    &format!("{}:{}", commit.hash, PLUGIN_STATS_PATH),
                ])
                .current_dir(
                    Path::new(OBS_RELEASES_REPO_PATH)
                        .canonicalize()
                        .expect("Failed to canonicalize path"),
                )
                .output()
                .expect("Failed to execute git command");

            let stats_str = String::from_utf8_lossy(&stats.stdout).to_string();
            let parsed_stats: Result<ObsDownloadStats, serde_json::Error> =
                serde_json::from_str(&stats_str);
            let result = match parsed_stats {
                Ok(stats) => Some(PluginDownloadStats::from_obs_data(stats, commit.clone())),
                Err(e) => {
                    eprintln!("Error parsing plugin download stats: {e}");
                    None
                }
            };

            let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
            if should_log_progress(done, total_commits) {
                println!("  Plugin download history progress: {done} / {total_commits}");
            }

            result
        })
        .collect()
}

fn filter_low_signal_plugins(plugin_data: Vec<BorrowedPluginData>) -> Vec<BorrowedPluginData> {
    let now = Date::now();

    plugin_data
        .into_iter()
        .filter(|plugin| {
            match &plugin.removed_commit {
                // remove plugins that were removed the day they were added
                Some(commit) => commit.date != plugin.added_commit.date,
                None => true,
            }
        })
        .filter(|plugin| {
            let mut advanced_date = plugin.added_commit.date.clone();
            advanced_date.advance_days(7);

            // remove plugins that have no downloads and are more than 7 days old
            !(plugin.download_count == 0 && advanced_date < now)
        })
        .collect()
}

pub fn build_plugin_stats() -> Result<(), Box<dyn std::error::Error>> {
    let time = std::time::Instant::now();
    let mut time2 = std::time::Instant::now();

    let plugin_lists = load_plugin_list_history();

    println!("Get plugin lists: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    let mut plugin_data = build_plugin_change_timeline(&plugin_lists);

    println!("Build Plugin Data {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    let download_stats = load_plugin_download_stat_history();

    println!("Get plugin download stats: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    backfill_download_history(&mut plugin_data, &download_stats);

    println!("Update weekly download stats: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    build_version_history(&mut plugin_data, &download_stats);

    println!("Update version history: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    plugin_data = filter_low_signal_plugins(plugin_data);

    write_in_chunks_atomic(Path::new(PLUGIN_DATA_PATH), &plugin_data, 50)?;

    println!("Filtered and write plugin data: {:#?}", time2.elapsed());

    println!("Plugin stats built in {:#?}", time.elapsed());

    Ok(())
}

pub fn read_plugin_data() -> Result<Vec<PluginData>, Box<dyn std::error::Error>> {
    read_chunked_data(Path::new(PLUGIN_DATA_PATH))
}
