use data_lib::{
    commit::Commit,
    date::Date,
    input_data::{ObsDownloadStats, ObsPluginList},
    plugin::PluginData,
};
use hashbrown::HashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{path::Path, process::Command};

use crate::{
    constants::{OBS_RELEASES_REPO_PATH, PLUGIN_DATA_PATH, PLUGIN_LIST_PATH, PLUGIN_STATS_PATH},
    file_utils::{empty_dir, read_chunked_data, write_in_chunks},
    plugins::{BorrowedPluginData, PluginDownloadStats, PluginList},
};

fn get_plugin_list_changes() -> Vec<Commit> {
    let git_output = Command::new("git")
        .args([
            "--no-pager",
            "log",
            "--diff-filter=M",
            "--date-order",
            "--reverse",
            "--format=\"%ad %H\"",
            "--date=iso-strict",
            "--grep=stats",
        ])
        .current_dir(
            Path::new(OBS_RELEASES_REPO_PATH)
                .canonicalize()
                .expect("Failed to canonicalize path"),
        )
        .output()
        .expect("Failed to execute git command");

    Commit::from_git_log(String::from_utf8_lossy(&git_output.stdout).to_string())
}

fn get_plugin_download_changes() -> Vec<Commit> {
    let git_output = Command::new("git")
        .args([
            "log",
            "--diff-filter=M",
            "--date-order",
            "--reverse",
            "--format=\"%ad %H\"",
            "--date=iso-strict",
            "--grep=stats",
        ])
        .current_dir(
            Path::new(OBS_RELEASES_REPO_PATH)
                .canonicalize()
                .expect("Failed to canonicalize path"),
        )
        .output()
        .expect("Failed to execute git command");

    Commit::from_git_log(String::from_utf8_lossy(&git_output.stdout).to_string())
}

fn get_plugin_lists() -> Vec<PluginList> {
    let commits = get_plugin_list_changes();

    assert!(!commits.is_empty(), "No plugin list changes found");

    commits
        .par_iter()
        .map(|commit| {
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
            let list: Result<ObsPluginList, serde_json::Error> = serde_json::from_str(&list_str);
            match list {
                Ok(list) => Some(PluginList {
                    entries: list.to_hashmap(),
                    commit: commit.clone(),
                }),
                Err(e) => {
                    eprintln!("Error parsing plugin list: {e}");
                    None
                }
            }
        })
        .filter_map(|x| x)
        .collect()
}

fn build_plugin_data(plugin_lists: &[PluginList]) -> Vec<BorrowedPluginData<'_>> {
    println!("Building plugin data...");

    let mut plugin_data_map = HashMap::new();

    assert!(!plugin_lists.is_empty(), "No plugin lists found");

    for (id, entry) in &plugin_lists[0].entries {
        plugin_data_map.insert(
            id.clone(),
            BorrowedPluginData::new(id.clone(), &plugin_lists[0].commit, entry),
        );
    }

    for plugin_list in plugin_lists.iter().skip(1) {
        // println!("Processing plugin list {} of {}", i, plugin_lists.len());

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
    }

    plugin_data_map.into_iter().map(|(_, data)| data).collect()
}

fn update_weekly_download_stats(
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

    // something in may 2024 is messed up, e.g. advanced-canvas
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
    ];

    for date in start_date.iterate_daily_to(&end_date) {
        if excluded_dates.contains(&date) {
            continue;
        }

        for entry in plugin_data.iter_mut() {
            // Don't update downloads for plugins that were not yet released
            if entry.added_commit.date > date {
                continue;
            }

            for i in 0..6 {
                let mut current_date = date.clone();
                current_date.advance_days(i);

                let download_stats = download_stats_map.get(&current_date);
                if let Some(stats) = download_stats {
                    entry.update_download_history(stats);
                    break;
                }
            }
        }
    }
}

fn update_version_history(
    plugin_data: &mut [BorrowedPluginData],
    download_stats: &[PluginDownloadStats],
) {
    println!("Updating version history...");

    for stat in download_stats {
        for entry in plugin_data.iter_mut() {
            entry.update_version_history(stat);
        }
    }

    for entry in plugin_data.iter_mut() {
        entry.sort_version_history();
    }
}

fn get_plugin_download_stats() -> Vec<PluginDownloadStats> {
    println!("Fetching plugin download stats...");

    let download_commits = get_plugin_download_changes();

    download_commits
        .par_iter()
        .map(|commit| {
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
            let stats: Result<ObsDownloadStats, serde_json::Error> =
                serde_json::from_str(&stats_str);
            match stats {
                Ok(stats) => Some(PluginDownloadStats::from_obs_data(stats, commit.clone())),
                Err(e) => {
                    eprintln!("Error parsing plugin download stats: {e}");
                    None
                }
            }
        })
        .filter_map(|x| x)
        .collect()
}

fn filter_plugins(plugin_data: Vec<BorrowedPluginData>) -> Vec<BorrowedPluginData> {
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
            !(plugin.download_count == 0 && advanced_date > now)
        })
        .collect()
}

pub fn build_plugin_stats() -> Result<(), Box<dyn std::error::Error>> {
    let time = std::time::Instant::now();
    let mut time2 = std::time::Instant::now();

    let plugin_lists = get_plugin_lists();

    println!("Get plugin lists: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    let mut plugin_data = build_plugin_data(&plugin_lists);

    println!("Build Plugin Data {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    let download_stats = get_plugin_download_stats();

    println!("Get plugin download stats: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    update_weekly_download_stats(&mut plugin_data, &download_stats);

    println!("Update weekly download stats: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    update_version_history(&mut plugin_data, &download_stats);

    println!("Update version history: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    plugin_data = filter_plugins(plugin_data);

    empty_dir(Path::new(PLUGIN_DATA_PATH))?;

    write_in_chunks(Path::new(PLUGIN_DATA_PATH), &plugin_data, 50)?;

    println!("Filtered and write plugin data: {:#?}", time2.elapsed());

    println!("Plugin stats built in {:#?}", time.elapsed());

    Ok(())
}

pub fn read_plugin_data() -> Result<Vec<PluginData>, Box<dyn std::error::Error>> {
    read_chunked_data(Path::new(PLUGIN_DATA_PATH))
}
