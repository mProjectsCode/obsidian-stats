use data_lib::{input_data::ObsThemeList, theme::ThemeData};
use hashbrown::HashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{path::Path, process::Command};

use crate::{
    constants::{OBS_RELEASES_REPO_PATH, THEME_DATA_PATH, THEME_LIST_PATH},
    file_utils::{empty_dir, read_chunked_data, write_in_chunks},
    git_utils::get_obs_repo_changes,
    theme::{BorrowedThemeData, ThemeIdCounter, ThemeList},
};

fn get_theme_lists() -> Vec<ThemeList> {
    let commits = get_obs_repo_changes();

    assert!(!commits.is_empty(), "No theme list changes found");

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
            let list: Result<ObsThemeList, serde_json::Error> = serde_json::from_str(&list_str);
            match list {
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
            }
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

    for theme_list in theme_lists.iter().skip(1) {
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

    empty_dir(Path::new(THEME_DATA_PATH))?;

    write_in_chunks(Path::new(THEME_DATA_PATH), &theme_data, 50)?;

    println!("Filtered and write theme data: {:#?}", time2.elapsed());

    println!("Theme stats built in {:#?}", time.elapsed());

    Ok(())
}

pub fn read_theme_data() -> Result<Vec<ThemeData>, Box<dyn std::error::Error>> {
    read_chunked_data(Path::new(THEME_DATA_PATH))
}
