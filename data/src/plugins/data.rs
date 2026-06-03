use data_lib::{
    date::Date,
    input_data::{ObsDownloadStats, ObsPluginList},
    plugin::PluginData,
};
use hashbrown::{HashMap, HashSet};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    error::Error,
    path::Path,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    constants::{OBS_RELEASES_REPO_PATH, PLUGIN_DATA_PATH, PLUGIN_LIST_PATH, PLUGIN_STATS_PATH},
    file_utils::{read_chunked_data, write_in_chunks_atomic},
    git_utils::get_obs_repo_changes_for_file,
    plugins::{BorrowedPluginData, PluginDownloadStats, PluginList, download_backfill},
    progress::should_log_progress,
};

fn load_plugin_list_history() -> Result<Vec<PluginList>, Box<dyn Error>> {
    let commits = get_obs_repo_changes_for_file(PLUGIN_LIST_PATH)?;
    let total_commits = commits.len();
    let obs_repo_path = Path::new(OBS_RELEASES_REPO_PATH).canonicalize()?;

    assert!(!commits.is_empty(), "No plugin list changes found");

    println!("Loading plugin list history from {total_commits} commits...");
    let processed = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);

    let results = commits
        .par_iter()
        .map(|commit| {
            let list = Command::new("git")
                .args([
                    "cat-file",
                    "-p",
                    &format!("{}:{}", commit.hash, PLUGIN_LIST_PATH),
                ])
                .current_dir(&obs_repo_path)
                .output()
                .map_err(|error| {
                    format!(
                        "failed to execute git cat-file for plugin list at {}: {error}",
                        commit.to_fancy_string()
                    )
                })?;

            if !list.status.success() {
                skipped.fetch_add(1, Ordering::Relaxed);
                return Ok(None);
            }

            let list_str = String::from_utf8_lossy(&list.stdout).to_string();
            if list_str.is_empty() {
                skipped.fetch_add(1, Ordering::Relaxed);
                return Ok(None);
            }
            let result = match serde_json::from_str::<ObsPluginList>(&list_str) {
                Ok(list) => PluginList {
                    entries: list.to_hashmap(),
                    commit: commit.clone(),
                },
                Err(_) => {
                    skipped.fetch_add(1, Ordering::Relaxed);
                    return Ok(None);
                }
            };

            let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
            if should_log_progress(done, total_commits) {
                println!("  Plugin list history progress: {done} / {total_commits}");
            }

            Ok(Some(result))
        })
        .collect::<Result<Vec<_>, String>>()
        .map_err(std::io::Error::other)?;

    let skipped = skipped.load(Ordering::Relaxed);
    if skipped > 0 {
        eprintln!("Warning: skipped {skipped} broken plugin list commit(s).");
    }

    let results = results.into_iter().flatten().collect();

    Ok(results)
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

fn build_version_history(
    plugin_data: &mut [BorrowedPluginData],
    download_stats: &[PluginDownloadStats],
) {
    println!("Updating version history...");

    let index_by_id = plugin_data
        .iter()
        .enumerate()
        .map(|(idx, entry)| (entry.id.clone(), idx))
        .collect::<HashMap<_, _>>();
    let mut previous_versions_by_plugin = vec![None::<HashSet<String>>; plugin_data.len()];

    let total_stats = download_stats.len();
    for (idx, stat) in download_stats.iter().enumerate() {
        let date = stat.get_date();
        if download_backfill::is_excluded_download_date(&date) {
            continue;
        }

        for (id, entry) in stat.entries.iter() {
            if let Some(&plugin_idx) = index_by_id.get(id) {
                let current_versions = valid_versions(&entry.versions);
                plugin_data[plugin_idx].update_version_history_from_snapshot(
                    &date,
                    previous_versions_by_plugin[plugin_idx].as_ref(),
                    &current_versions,
                );
                previous_versions_by_plugin[plugin_idx] = Some(current_versions);
            }
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

fn valid_versions(versions: &[String]) -> HashSet<String> {
    versions
        .iter()
        .filter(|version| data_lib::version::Version::validate(version))
        .cloned()
        .collect()
}

fn load_plugin_download_stat_history() -> Result<Vec<PluginDownloadStats>, Box<dyn Error>> {
    println!("Fetching plugin download stats...");

    let commits = get_obs_repo_changes_for_file(PLUGIN_STATS_PATH)?;
    let total_commits = commits.len();
    let obs_repo_path = Path::new(OBS_RELEASES_REPO_PATH).canonicalize()?;
    println!("Loading plugin download stats from {total_commits} commits...");
    let processed = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);

    let results = commits
        .par_iter()
        .map(|commit| {
            let stats = Command::new("git")
                .args([
                    "cat-file",
                    "-p",
                    &format!("{}:{}", commit.hash, PLUGIN_STATS_PATH),
                ])
                .current_dir(&obs_repo_path)
                .output()
                .map_err(|error| {
                    format!(
                        "failed to execute git cat-file for plugin download stats at {}: {error}",
                        commit.to_fancy_string()
                    )
                })?;

            if !stats.status.success() {
                skipped.fetch_add(1, Ordering::Relaxed);
                return Ok(None);
            }

            let stats_str = String::from_utf8_lossy(&stats.stdout).to_string();
            let result = match serde_json::from_str::<ObsDownloadStats>(&stats_str) {
                Ok(stats) => PluginDownloadStats::from_obs_data(stats, commit.clone()),
                Err(_) => {
                    skipped.fetch_add(1, Ordering::Relaxed);
                    return Ok(None);
                }
            };

            let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
            if should_log_progress(done, total_commits) {
                println!("  Plugin download history progress: {done} / {total_commits}");
            }

            Ok(Some(result))
        })
        .collect::<Result<Vec<_>, String>>()
        .map_err(std::io::Error::other)?;

    let skipped = skipped.load(Ordering::Relaxed);
    if skipped > 0 {
        eprintln!("Warning: skipped {skipped} broken plugin download stats commit(s).");
    }

    let results = results.into_iter().flatten().collect();

    Ok(results)
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

    let plugin_lists = load_plugin_list_history()?;

    println!("Get plugin lists: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    let mut plugin_data = build_plugin_change_timeline(&plugin_lists);

    println!("Build Plugin Data {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    let download_stats = load_plugin_download_stat_history()?;

    println!("Get plugin download stats: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    download_backfill::backfill_download_history(&mut plugin_data, &download_stats);

    println!("Update weekly download stats: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    build_version_history(&mut plugin_data, &download_stats);

    println!("Update version history: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    plugin_data = filter_low_signal_plugins(plugin_data);
    plugin_data.sort_by(|a, b| a.id.cmp(&b.id));

    write_in_chunks_atomic(Path::new(PLUGIN_DATA_PATH), &plugin_data, 50)?;

    println!("Filtered and write plugin data: {:#?}", time2.elapsed());

    println!("Plugin stats built in {:#?}", time.elapsed());

    Ok(())
}

pub fn read_plugin_data() -> Result<Vec<PluginData>, Box<dyn std::error::Error>> {
    read_chunked_data(Path::new(PLUGIN_DATA_PATH))
}
