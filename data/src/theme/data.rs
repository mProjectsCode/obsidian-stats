use data_lib::{input_data::ObsThemeList, theme::ThemeData};
use hashbrown::HashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    path::Path,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    constants::{OBS_RELEASES_REPO_PATH, THEME_DATA_PATH, THEME_LIST_PATH},
    file_utils::{read_chunked_data, write_in_chunks_atomic},
    git_utils::get_obs_repo_changes,
    progress::should_log_progress,
    theme::{BorrowedThemeData, ThemeIdCounter, ThemeList},
};

fn get_theme_lists() -> Vec<ThemeList> {
    let commits = get_obs_repo_changes();
    let total_commits = commits.len();

    assert!(!commits.is_empty(), "No theme list changes found");

    println!("Loading theme list history from {total_commits} commits...");
    let processed = AtomicUsize::new(0);

    commits
        .par_iter()
        .filter_map(|commit| {
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
            if list_str.is_empty() {
                eprintln!("Empty theme list at commit {}", commit.to_fancy_string());
                return None;
            }
            let parsed_list: Result<ObsThemeList, serde_json::Error> =
                serde_json::from_str(&list_str);
            let result = match parsed_list {
                Ok(list) => Some(ThemeList {
                    entries: list.to_hashmap(),
                    commit: commit.clone(),
                }),
                Err(e) => {
                    eprintln!(
                        "Error parsing theme list at commit {}: {}",
                        commit.to_fancy_string(),
                        e
                    );
                    None
                }
            };

            let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
            if should_log_progress(done, total_commits) {
                println!("  Theme list history progress: {done} / {total_commits}");
            }

            result
        })
        .collect()
}

fn build_theme_data(theme_lists: &[ThemeList]) -> Vec<BorrowedThemeData<'_>> {
    println!("Building theme data...");

    let mut theme_data_map = HashMap::new();

    assert!(!theme_lists.is_empty(), "No theme lists found");

    let mut id_counter = ThemeIdCounter::new();

    for (id, entry) in &theme_lists[0].entries {
        theme_data_map.insert(
            id.clone(),
            BorrowedThemeData::new(id.clone(), &theme_lists[0].commit, entry, &mut id_counter),
        );
    }

    let total_lists = theme_lists.len();
    for (idx, theme_list) in theme_lists.iter().enumerate().skip(1) {
        for (_, theme) in theme_data_map.iter_mut() {
            theme.find_changes(theme_list);
        }

        for (id, entry) in &theme_list.entries {
            if !theme_data_map.contains_key(id) {
                theme_data_map.insert(
                    id.clone(),
                    BorrowedThemeData::new(id.clone(), &theme_list.commit, entry, &mut id_counter),
                );
            }
        }

        if should_log_progress(idx + 1, total_lists) {
            println!("  Theme timeline progress: {} / {}", idx + 1, total_lists);
        }
    }

    theme_data_map.into_iter().map(|(_, data)| data).collect()
}

fn filter_themes(theme_data: Vec<BorrowedThemeData>) -> Vec<BorrowedThemeData> {
    theme_data
        .into_iter()
        .filter(|theme| {
            match &theme.removed_commit {
                // remove themes that were removed the day they were added
                Some(commit) => commit.date != theme.added_commit.date,
                None => true,
            }
        })
        .collect()
}

pub fn build_theme_stats() -> Result<(), Box<dyn std::error::Error>> {
    let time = std::time::Instant::now();
    let mut time2 = std::time::Instant::now();

    let theme_lists = get_theme_lists();

    println!("Get theme lists: {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    let mut theme_data = build_theme_data(&theme_lists);

    println!("Build theme Data {:#?}", time2.elapsed());
    time2 = std::time::Instant::now();

    theme_data = filter_themes(theme_data);

    write_in_chunks_atomic(Path::new(THEME_DATA_PATH), &theme_data, 50)?;

    println!("Filtered and write theme data: {:#?}", time2.elapsed());

    println!("Theme stats built in {:#?}", time.elapsed());

    Ok(())
}

pub fn read_theme_data() -> Result<Vec<ThemeData>, Box<dyn std::error::Error>> {
    read_chunked_data(Path::new(THEME_DATA_PATH))
}
