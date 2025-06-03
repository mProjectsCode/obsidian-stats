use hashbrown::HashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{path::Path, process::Command};

use crate::{
    commit::Commit,
    constants::{OBS_RELEASES_REPO_PATH, THEME_DATA_PATH, THEME_LIST_PATH, THEME_STATS_PATH},
    date::Date,
    input_data::{ObsThemeList},
    theme::{ThemeData, ThemeDownloadStats, ThemeList, SerializedThemeData},
};

fn get_theme_list_changes() -> Vec<Commit> {
    let git_output = Command::new("git")
        .args([
            "--no-pager",
            "log",
            "--diff-filter=M",
            "--date-order",
            "--reverse",
            "--format=\"%ad %H\"",
            "--date=iso-strict",
            // "--grep=plugin stats",
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

fn get_theme_lists() -> Vec<ThemeList> {
    let commits = get_theme_list_changes();

    assert!(!commits.is_empty(), "No theme list changes found");

    commits
        .par_iter()
        .map(|commit| {
            let list = Command::new("git")
                .args([
                    "cat-file",
                    "-p",
                    &format!("{}:{}", commit.hash, THEME_LIST_PATH),
                ])
                .current_dir(
                    Path::new(OBS_RELEASES_REPO_PATH)
                        .canonicalize()
                        .expect("Failed to canonicalize path"),
                )
                .output()
                .expect("Failed to execute git command");

            let list_str = String::from_utf8_lossy(&list.stdout).to_string();
            let list: Result<ObsThemeList, serde_json::Error> = serde_json::from_str(&list_str);
            match list {
                Ok(list) => Some(ThemeList {
                    entries: list.to_hashmap(),
                    commit: commit.clone(),
                }),
                Err(e) => {
                    eprintln!("Error parsing plugin list: {}", e);
                    None
                }
            }
        })
        .filter_map(|x| x)
        .collect()
}

fn build_theme_data(theme_lists: &[ThemeList]) -> Vec<ThemeData> {
    println!("Building theme data...");

    let mut theme_data_map = HashMap::new();

    assert!(!theme_lists.is_empty(), "No plugin lists found");

    for (id, entry) in &theme_lists[0].entries {
        theme_data_map.insert(
            id.clone(),
            PluginData::new(id.clone(), &theme_lists[0].commit, entry),
        );
    }

    for theme_list in theme_lists.iter().skip(1) {
        // println!("Processing plugin list {} of {}", i, theme_lists.len());

        for (_, theme) in theme_data_map.iter_mut() {
            theme.find_changes(theme_list);
        }

        for (id, entry) in &theme_list.entries {
            if !theme_data_map.contains_key(id) {
                theme_data_map.insert(
                    id.clone(),
                    PluginData::new(id.clone(), &theme_list.commit, entry),
                );
            }
        }
    }

    return theme_data_map.into_iter().map(|(_, data)| data).collect();
}

fn update_version_history(plugin_data: &mut [PluginData], download_stats: &[PluginDownloadStats]) {
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
                    eprintln!("Error parsing plugin download stats: {}", e);
                    None
                }
            }
        })
        .filter_map(|x| x)
        .collect()
}

fn filter_plugins(plugin_data: Vec<PluginData>) -> Vec<PluginData> {
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

pub fn build_plugin_stats() {
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

    std::fs::write(
        Path::new(PLUGIN_DATA_PATH),
        serde_json::to_string_pretty(&plugin_data)
            .expect("Failed to serialize plugin data to JSON"),
    )
    .expect("Failed to write plugin data to file");

    println!("Filtered and write plugin data: {:#?}", time2.elapsed());

    println!("Plugin stats built in {:#?}", time.elapsed());
}

pub fn read_plugin_data() -> Vec<SerializedPluginData> {
    let data = std::fs::read_to_string(Path::new(PLUGIN_DATA_PATH))
        .expect("Failed to read plugin data file");

    serde_json::from_str(&data).expect("Failed to parse plugin data JSON")
}