use std::{num::NonZero, path::Path};

use data_lib::plugin::PluginData;
use gix::prepare_clone;
use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelIterator, ParallelIterator},
};
use serde::{Deserialize, Serialize};

use hashbrown::HashMap;

use crate::{
    constants::{CLONE_STATE_PATH, DEFAULT_CLONE_REFRESH_DAYS, PLUGIN_REPO_PATH},
    file_utils::ensure_dir,
    plugins::{data::read_plugin_data, release_acquisition::acquire_plugin_release_main_js},
    state::{is_fresh, now_unix_seconds, read_json_or_default, write_json_atomic},
};

enum CloneResult {
    Success(String),
    Skipped(String),
    Failed(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CloneState {
    entries: HashMap<String, CloneStateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CloneStateEntry {
    repo: String,
    last_attempt_unix: i64,
    last_success_unix: Option<i64>,
    status: String,
}

pub fn clone_plugin_repos() -> Result<(), Box<dyn std::error::Error>> {
    ensure_dir(Path::new(PLUGIN_REPO_PATH))?;

    println!("Loading data...");

    let data = read_plugin_data()?;
    let refresh_days = std::env::var("CLONE_REFRESH_DAYS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(DEFAULT_CLONE_REFRESH_DAYS);

    let mut state: CloneState = read_json_or_default(Path::new(CLONE_STATE_PATH));

    println!(
        "Starting cloning process (refresh window: {} days)...",
        refresh_days
    );

    let clone_jobs = data
        .iter()
        .filter(|plugin| {
            if plugin.removed_commit.is_some() {
                return true;
            }

            let path = Path::new(PLUGIN_REPO_PATH).join(plugin.id.clone());
            let is_existing = path.exists();
            if !is_existing {
                return true;
            }

            let state_entry = state.entries.get(&plugin.id);
            if let Some(state_entry) = state_entry
                && state_entry.repo == plugin.current_entry.repo
                && state_entry
                    .last_success_unix
                    .is_some_and(|t| is_fresh(t, refresh_days))
            {
                return false;
            }

            true
        })
        .cloned()
        .collect::<Vec<_>>();

    let fresh_skipped = data.len().saturating_sub(clone_jobs.len());
    println!(
        "Clone plan: total={}, queued={}, fresh_skipped={}",
        data.len(),
        clone_jobs.len(),
        fresh_skipped
    );

    let now = std::time::Instant::now();

    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .expect("Failed to build thread pool");

    let clone_results: Vec<_> = thread_pool.install(|| {
        clone_jobs
            .into_par_iter()
            .map(|plugin| {
                if plugin.removed_commit.is_some() {
                    return CloneResult::Skipped(plugin.id);
                }

                let target_path = Path::new(PLUGIN_REPO_PATH).join(plugin.id.clone());
                if target_path.exists() {
                    let _ = std::fs::remove_dir_all(&target_path);
                }

                let clone_task = prepare_shallow_clone(&plugin);
                let mut clone_task = match clone_task {
                    Ok(task) => task,
                    Err(e) => {
                        return CloneResult::Failed(plugin.id, e.to_string());
                    }
                };

                let clone_res: Result<_, String> = clone_task
                    .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
                    .map_err(|e| e.to_string())
                    .and_then(|(mut checkout, _)| {
                        checkout
                            .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
                            .map_err(|e| e.to_string())
                    });
                match clone_res {
                    Ok(_) => CloneResult::Success(plugin.id),
                    Err(e) => CloneResult::Failed(plugin.id, e),
                }
            })
            .collect()
    });

    for result in &clone_results {
        match result {
            CloneResult::Success(id) => {
                let repo = data
                    .iter()
                    .find(|p| &p.id == id)
                    .map(|p| p.current_entry.repo.clone())
                    .unwrap_or_default();
                state.entries.insert(
                    id.clone(),
                    CloneStateEntry {
                        repo,
                        last_attempt_unix: now_unix_seconds(),
                        last_success_unix: Some(now_unix_seconds()),
                        status: "ok".to_string(),
                    },
                );
            }
            CloneResult::Failed(id, err) => {
                let repo = data
                    .iter()
                    .find(|p| &p.id == id)
                    .map(|p| p.current_entry.repo.clone())
                    .unwrap_or_default();
                state.entries.insert(
                    id.clone(),
                    CloneStateEntry {
                        repo,
                        last_attempt_unix: now_unix_seconds(),
                        last_success_unix: None,
                        status: format!("failed:{err}"),
                    },
                );
            }
            CloneResult::Skipped(id) => {
                let repo = data
                    .iter()
                    .find(|p| &p.id == id)
                    .map(|p| p.current_entry.repo.clone())
                    .unwrap_or_default();
                state.entries.insert(
                    id.clone(),
                    CloneStateEntry {
                        repo,
                        last_attempt_unix: now_unix_seconds(),
                        last_success_unix: None,
                        status: "skipped".to_string(),
                    },
                );
            }
        }
    }

    write_json_atomic(Path::new(CLONE_STATE_PATH), &state)?;

    acquire_plugin_release_main_js(&data)?;

    let failed_plugins: Vec<_> = clone_results
        .iter()
        .filter_map(|result| match result {
            CloneResult::Failed(id, error) => Some((id, error)),
            _ => None,
        })
        .collect();

    let skipped_plugins: Vec<_> = clone_results
        .iter()
        .filter_map(|result| match result {
            CloneResult::Skipped(id) => Some(id),
            _ => None,
        })
        .collect();

    let success_count = clone_results
        .iter()
        .filter(|result| matches!(result, CloneResult::Success(_)))
        .count();

    println!("Clone summary:");
    println!("  Success: {}", success_count);
    println!("  Skipped (removed): {}", skipped_plugins.len());
    println!("  Skipped (fresh): {}", fresh_skipped);
    println!("  Failed: {}", failed_plugins.len());
    println!();

    for (id, error) in failed_plugins {
        eprintln!("Failed to clone plugin {id}: {error}");
    }

    println!("Cloning completed in {:?}", now.elapsed());

    Ok(())
}

fn prepare_shallow_clone(
    plugin: &PluginData,
) -> Result<gix::clone::PrepareFetch, Box<dyn std::error::Error>> {
    let clone = prepare_clone(
        gix::url::parse(
            format!("https://github.com/{}.git", plugin.current_entry.repo)
                .as_str()
                .into(),
        )?,
        Path::new(PLUGIN_REPO_PATH).join(plugin.id.clone()),
    )?
    .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(
        NonZero::new(1).unwrap(),
    ))
    .configure_remote(|remote| Ok(remote.with_fetch_tags(gix::remote::fetch::Tags::None)));

    Ok(clone)
}
