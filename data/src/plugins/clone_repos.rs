use std::sync::atomic::{AtomicUsize, Ordering};
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
    alerts,
    constants::{CLONE_STATE_PATH, DEFAULT_CLONE_REFRESH_DAYS, PLUGIN_REPO_PATH},
    file_utils::ensure_dir,
    plugins::{data::read_plugin_data, release_acquisition::acquire_plugin_release_main_js},
    progress::should_log_progress,
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
    let total_jobs = clone_jobs.len();
    let processed = AtomicUsize::new(0);

    let clone_results: Vec<_> = thread_pool.install(|| {
        clone_jobs
            .into_par_iter()
            .map(|plugin| {
                let result = if plugin.removed_commit.is_some() {
                    CloneResult::Skipped(plugin.id)
                } else {
                    clone_repo_preserving_previous(&plugin)
                };

                let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
                if should_log_progress(done, total_jobs) {
                    println!("  Clone progress: {done} / {total_jobs}");
                }

                result
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

    for (id, error) in &failed_plugins {
        eprintln!("Failed to clone plugin {id}: {error}");
    }
    if !failed_plugins.is_empty() {
        let details = failed_plugins
            .iter()
            .take(10)
            .map(|(id, error)| format!("{id}: {error}"))
            .collect::<Vec<_>>()
            .join("; ");
        alerts::record_unexpected_error(
            "plugin repository cloning",
            format!("{} clone(s) failed. {}", failed_plugins.len(), details),
        );
    }

    println!("Cloning completed in {:?}", now.elapsed());

    Ok(())
}

fn prepare_shallow_clone(
    plugin: &PluginData,
    target_path: &Path,
) -> Result<gix::clone::PrepareFetch, Box<dyn std::error::Error>> {
    let clone = prepare_clone(
        gix::url::parse(
            format!("https://github.com/{}.git", plugin.current_entry.repo)
                .as_str()
                .into(),
        )?,
        target_path,
    )?
    .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(
        NonZero::new(1).unwrap(),
    ))
    .configure_remote(|remote| Ok(remote.with_fetch_tags(gix::remote::fetch::Tags::None)));

    Ok(clone)
}

fn clone_repo_preserving_previous(plugin: &PluginData) -> CloneResult {
    let target_path = Path::new(PLUGIN_REPO_PATH).join(&plugin.id);
    let timestamp = now_unix_seconds();
    let tmp_path = Path::new(PLUGIN_REPO_PATH).join(format!(".tmp-{}-{timestamp}", plugin.id));
    let backup_path = Path::new(PLUGIN_REPO_PATH).join(format!(".bak-{}-{timestamp}", plugin.id));

    if tmp_path.exists() {
        let _ = std::fs::remove_dir_all(&tmp_path);
    }
    if backup_path.exists() {
        let _ = std::fs::remove_dir_all(&backup_path);
    }

    let mut clone_task = match prepare_shallow_clone(plugin, &tmp_path) {
        Ok(task) => task,
        Err(error) => return CloneResult::Failed(plugin.id.clone(), error.to_string()),
    };

    let clone_result: Result<_, String> = clone_task
        .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        .map_err(|e| e.to_string())
        .and_then(|(mut checkout, _)| {
            checkout
                .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
                .map_err(|e| e.to_string())
        });

    if let Err(error) = clone_result {
        let _ = std::fs::remove_dir_all(&tmp_path);
        return CloneResult::Failed(plugin.id.clone(), error);
    }

    let had_existing_target = target_path.exists();
    if had_existing_target && let Err(error) = std::fs::rename(&target_path, &backup_path) {
        let _ = std::fs::remove_dir_all(&tmp_path);
        return CloneResult::Failed(plugin.id.clone(), error.to_string());
    }

    match std::fs::rename(&tmp_path, &target_path) {
        Ok(()) => {
            if had_existing_target {
                let _ = std::fs::remove_dir_all(&backup_path);
            }
            CloneResult::Success(plugin.id.clone())
        }
        Err(error) => {
            if had_existing_target {
                let _ = std::fs::rename(&backup_path, &target_path);
            }
            let _ = std::fs::remove_dir_all(&tmp_path);
            CloneResult::Failed(plugin.id.clone(), error.to_string())
        }
    }
}
