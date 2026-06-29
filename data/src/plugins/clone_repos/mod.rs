use std::{
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use data_lib::{latest_data_update::PluginPageCloneFreshness, plugin::PluginData};
use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelIterator, ParallelIterator},
};
use serde::{Deserialize, Serialize};

use hashbrown::HashMap;

mod git_clone;

use git_clone::{CloneResult, clone_repo_preserving_previous};

use crate::{
    alerts,
    constants::{
        CLONE_STATE_PATH, DEFAULT_CLONE_TIMEOUT_SECONDS, DEFAULT_MAX_CLONE_THREADS,
        PLUGIN_REPO_PATH,
    },
    file_utils::ensure_dir,
    plugins::{data::read_plugin_data, stats_helper::HelperPluginStore},
    progress::should_log_progress,
    security::validated_plugin_path,
    state::{now_unix_seconds, read_json_or_default, write_json_atomic},
};

const CLONE_THREADS_ENV: &str = "CLONE_THREADS";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CloneStatus {
    Ok,
    SkippedRemoved,
}

impl CloneStatus {
    fn as_state_value(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::SkippedRemoved => "skipped_removed",
        }
    }
}

struct CloneJob {
    plugin: PluginData,
    target_release_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CloneState {
    entries: HashMap<String, CloneStateEntry>,
}

type CloneStateEntry = PluginPageCloneFreshness;

pub fn clone_plugin_repos(force: bool, no_clone: bool) -> Result<(), Box<dyn std::error::Error>> {
    ensure_dir(Path::new(PLUGIN_REPO_PATH))?;

    println!("Loading data...");

    let data = read_plugin_data()?;
    let clone_timeout = Duration::from_secs(
        std::env::var("CLONE_TIMEOUT_SECONDS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_CLONE_TIMEOUT_SECONDS),
    );
    let default_threads = std::thread::available_parallelism()
        .map(|n| default_clone_thread_count(n.get()))
        .unwrap_or(4);
    let thread_count = configured_thread_count(CLONE_THREADS_ENV, default_threads);

    let mut state: CloneState = read_json_or_default(Path::new(CLONE_STATE_PATH));
    let run_started_unix = now_unix_seconds();
    let helper_store = HelperPluginStore::read()?;

    println!(
        "Starting cloning process (clone timeout: {}s, force: {}, no_clone: {}, threads: {})...",
        clone_timeout.as_secs(),
        force,
        no_clone,
        thread_count
    );

    if no_clone {
        println!("Skipping repository recloning because --no-clone was set.");
        return Ok(());
    }

    let mut clone_jobs = Vec::new();
    let mut skipped_removed = 0;
    let mut skipped_missing_version = 0;
    let mut skipped_current = 0;

    for plugin in &data {
        if plugin.removed_commit.is_some() {
            skipped_removed += 1;
            state.entries.insert(
                plugin.id.clone(),
                CloneStateEntry {
                    repo: plugin.current_entry.repo.clone(),
                    target_release_tag: None,
                    last_attempt_unix: run_started_unix,
                    last_success_unix: None,
                    status: CloneStatus::SkippedRemoved.as_state_value().to_string(),
                },
            );
            continue;
        }

        let target_release_tag = match helper_store.target_release_for_plugin(plugin) {
            Ok(target) => target.tag,
            Err(error) => {
                skipped_missing_version += 1;
                let previous_success = state.entries.get(&plugin.id).and_then(|entry| {
                    if entry.repo == plugin.current_entry.repo {
                        entry.last_success_unix
                    } else {
                        None
                    }
                });
                state.entries.insert(
                    plugin.id.clone(),
                    CloneStateEntry {
                        repo: plugin.current_entry.repo.clone(),
                        target_release_tag: None,
                        last_attempt_unix: run_started_unix,
                        last_success_unix: previous_success,
                        status: error.as_state_value().to_string(),
                    },
                );
                continue;
            }
        };

        let path = validated_plugin_path(Path::new(PLUGIN_REPO_PATH), &plugin.id);
        let state_entry = state.entries.get(&plugin.id);
        if path.as_ref().is_ok_and(|path| path.exists())
            && !force
            && let Some(state_entry) = state_entry
            && state_entry.repo == plugin.current_entry.repo
            && state_entry.target_release_tag.as_deref() == Some(target_release_tag.as_str())
            && state_entry.status == CloneStatus::Ok.as_state_value()
        {
            skipped_current += 1;
            continue;
        }

        clone_jobs.push(CloneJob {
            plugin: plugin.clone(),
            target_release_tag,
        });
    }

    println!(
        "Clone plan: total={}, queued={}, current_skipped={}, removed_skipped={}, missing_version_skipped={}",
        data.len(),
        clone_jobs.len(),
        skipped_current,
        skipped_removed,
        skipped_missing_version
    );

    let now = std::time::Instant::now();

    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .expect("Failed to build thread pool");
    let total_jobs = clone_jobs.len();
    let started = AtomicUsize::new(0);
    let processed = AtomicUsize::new(0);

    let clone_results: Vec<_> = thread_pool.install(|| {
        clone_jobs
            .into_par_iter()
            .map(|job| {
                let job_number = started.fetch_add(1, Ordering::Relaxed) + 1;
                let job_started = Instant::now();
                println!(
                    "  Clone start: {job_number} / {total_jobs} {} ({}, tag {})",
                    job.plugin.id, job.plugin.current_entry.repo, job.target_release_tag
                );
                let result = clone_repo_preserving_previous(
                    &job.plugin,
                    &job.target_release_tag,
                    clone_timeout,
                );

                let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
                let elapsed = job_started.elapsed();
                match &result {
                    CloneResult::Success { id, .. } => {
                        println!(
                            "  Clone done: {done} / {total_jobs} {id} ok in {:.1}s",
                            elapsed.as_secs_f32()
                        );
                    }
                    CloneResult::Failed { id, error, .. } => {
                        println!(
                            "  Clone done: {done} / {total_jobs} {id} failed in {:.1}s: {error}",
                            elapsed.as_secs_f32()
                        );
                    }
                }
                if should_log_progress(done, total_jobs) && done != total_jobs {
                    println!("  Clone progress checkpoint: {done} / {total_jobs}");
                }

                result
            })
            .collect()
    });

    let state_updated_unix = now_unix_seconds();
    for result in &clone_results {
        match result {
            CloneResult::Success {
                id,
                repo,
                target_release_tag,
            } => {
                state.entries.insert(
                    id.clone(),
                    CloneStateEntry {
                        repo: repo.clone(),
                        target_release_tag: Some(target_release_tag.clone()),
                        last_attempt_unix: state_updated_unix,
                        last_success_unix: Some(state_updated_unix),
                        status: CloneStatus::Ok.as_state_value().to_string(),
                    },
                );
            }
            CloneResult::Failed {
                id,
                repo,
                target_release_tag,
                error,
            } => {
                let previous_success = state.entries.get(id).and_then(|entry| {
                    if entry.repo == *repo {
                        entry.last_success_unix
                    } else {
                        None
                    }
                });
                state.entries.insert(
                    id.clone(),
                    CloneStateEntry {
                        repo: repo.clone(),
                        target_release_tag: Some(target_release_tag.clone()),
                        last_attempt_unix: state_updated_unix,
                        last_success_unix: previous_success,
                        status: error.as_state_value(),
                    },
                );
            }
        }
    }

    write_json_atomic(Path::new(CLONE_STATE_PATH), &state)?;

    let failed_plugins: Vec<_> = clone_results
        .iter()
        .filter_map(|result| match result {
            CloneResult::Failed { id, error, .. } => Some((id, error)),
            _ => None,
        })
        .collect();

    let success_count = clone_results
        .iter()
        .filter(|result| matches!(result, CloneResult::Success { .. }))
        .count();

    println!("Clone summary:");
    println!("  Success: {}", success_count);
    println!("  Skipped (removed): {}", skipped_removed);
    println!("  Skipped (current tag): {}", skipped_current);
    println!("  Skipped (missing version): {}", skipped_missing_version);
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

fn configured_thread_count(env_var: &str, default_threads: usize) -> usize {
    std::env::var(env_var)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|count| *count > 0)
        .unwrap_or(default_threads)
}

fn default_clone_thread_count(available_threads: usize) -> usize {
    available_threads.min(DEFAULT_MAX_CLONE_THREADS)
}

#[cfg(test)]
mod tests {
    use super::{CloneStatus, default_clone_thread_count};

    #[test]
    fn clone_status_values_match_persisted_labels() {
        assert_eq!(CloneStatus::Ok.as_state_value(), "ok");
        assert_eq!(
            CloneStatus::SkippedRemoved.as_state_value(),
            "skipped_removed"
        );
    }

    #[test]
    fn default_clone_concurrency_is_bounded() {
        assert_eq!(default_clone_thread_count(4), 4);
        assert_eq!(default_clone_thread_count(64), 8);
    }
}
