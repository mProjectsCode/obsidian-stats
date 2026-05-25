use std::{
    fmt,
    path::Path,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};

use data_lib::plugin::PluginData;
use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelIterator, ParallelIterator},
};
use serde::{Deserialize, Serialize};

use hashbrown::HashMap;

use crate::{
    alerts,
    constants::{CLONE_STATE_PATH, DEFAULT_CLONE_TIMEOUT_SECONDS, PLUGIN_REPO_PATH},
    file_utils::ensure_dir,
    plugins::{
        data::read_plugin_data,
        release_acquisition::{acquire_plugin_release_main_js, latest_version_from_history},
    },
    progress::should_log_progress,
    state::{now_unix_seconds, read_json_or_default, write_json_atomic},
};

enum CloneResult {
    Success {
        id: String,
        repo: String,
        target_release_tag: String,
    },
    Failed {
        id: String,
        repo: String,
        target_release_tag: String,
        error: CloneError,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CloneStatus {
    Ok,
    SkippedRemoved,
    VersionHistoryMissing,
}

impl CloneStatus {
    fn as_state_value(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::SkippedRemoved => "skipped_removed",
            Self::VersionHistoryMissing => "version_history_missing",
        }
    }
}

#[derive(Debug, Clone)]
enum CloneError {
    GitStart(String),
    GitOutput(String),
    GitFailed(String),
    GitTimeout {
        seconds: u64,
        detail: Option<String>,
    },
    GitWait(String),
    MoveExistingToBackup(String),
    MoveCloneIntoPlace {
        error: String,
        restore_error: Option<String>,
    },
}

impl CloneError {
    fn as_state_value(&self) -> String {
        format!("failed:{self}")
    }
}

impl fmt::Display for CloneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitStart(error) => write!(f, "failed to start git clone: {error}"),
            Self::GitOutput(error) => write!(f, "failed to read git clone output: {error}"),
            Self::GitFailed(error) => f.write_str(error),
            Self::GitTimeout { seconds, detail } => {
                write!(f, "timed out after {seconds}s")?;
                if let Some(detail) = detail
                    && !detail.is_empty()
                {
                    write!(f, ": {detail}")?;
                }
                Ok(())
            }
            Self::GitWait(error) => write!(f, "failed while waiting for git clone: {error}"),
            Self::MoveExistingToBackup(error) => {
                write!(f, "failed to move existing clone to backup: {error}")
            }
            Self::MoveCloneIntoPlace {
                error,
                restore_error,
            } => {
                write!(f, "failed to move cloned repo into place: {error}")?;
                if let Some(restore_error) = restore_error {
                    write!(f, "; failed to restore previous clone: {restore_error}")?;
                }
                Ok(())
            }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CloneStateEntry {
    repo: String,
    target_release_tag: Option<String>,
    last_attempt_unix: i64,
    last_success_unix: Option<i64>,
    status: String,
}

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

    let mut state: CloneState = read_json_or_default(Path::new(CLONE_STATE_PATH));
    let run_started_unix = now_unix_seconds();

    println!(
        "Starting cloning process (clone timeout: {}s, force: {}, no_clone: {})...",
        clone_timeout.as_secs(),
        force,
        no_clone
    );

    if no_clone {
        println!("Skipping repository recloning because --no-clone was set.");
        acquire_plugin_release_main_js(&data, force)?;
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

        let target_release_tag = match latest_version_from_history(plugin) {
            Some(version) => version,
            None => {
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
                        status: CloneStatus::VersionHistoryMissing
                            .as_state_value()
                            .to_string(),
                    },
                );
                continue;
            }
        };

        let path = Path::new(PLUGIN_REPO_PATH).join(&plugin.id);
        let state_entry = state.entries.get(&plugin.id);
        if path.exists()
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
        .num_threads(4)
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

    acquire_plugin_release_main_js(&data, force)?;

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

fn clone_repo_preserving_previous(
    plugin: &PluginData,
    target_release_tag: &str,
    clone_timeout: Duration,
) -> CloneResult {
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

    if let Err(error) = run_git_clone(plugin, target_release_tag, &tmp_path, clone_timeout) {
        let _ = std::fs::remove_dir_all(&tmp_path);
        return CloneResult::Failed {
            id: plugin.id.clone(),
            repo: plugin.current_entry.repo.clone(),
            target_release_tag: target_release_tag.to_string(),
            error,
        };
    }

    let had_existing_target = target_path.exists();
    if had_existing_target && let Err(error) = std::fs::rename(&target_path, &backup_path) {
        let _ = std::fs::remove_dir_all(&tmp_path);
        return CloneResult::Failed {
            id: plugin.id.clone(),
            repo: plugin.current_entry.repo.clone(),
            target_release_tag: target_release_tag.to_string(),
            error: CloneError::MoveExistingToBackup(error.to_string()),
        };
    }

    match std::fs::rename(&tmp_path, &target_path) {
        Ok(()) => {
            if had_existing_target {
                let _ = std::fs::remove_dir_all(&backup_path);
            }
            CloneResult::Success {
                id: plugin.id.clone(),
                repo: plugin.current_entry.repo.clone(),
                target_release_tag: target_release_tag.to_string(),
            }
        }
        Err(error) => {
            let restore_error = if had_existing_target {
                std::fs::rename(&backup_path, &target_path)
                    .err()
                    .map(|error| error.to_string())
            } else {
                None
            };
            if restore_error.is_none() {
                let _ = std::fs::remove_dir_all(&backup_path);
            }
            let _ = std::fs::remove_dir_all(&tmp_path);
            CloneResult::Failed {
                id: plugin.id.clone(),
                repo: plugin.current_entry.repo.clone(),
                target_release_tag: target_release_tag.to_string(),
                error: CloneError::MoveCloneIntoPlace {
                    error: error.to_string(),
                    restore_error,
                },
            }
        }
    }
}

fn run_git_clone(
    plugin: &PluginData,
    target_release_tag: &str,
    tmp_path: &Path,
    clone_timeout: Duration,
) -> Result<(), CloneError> {
    let repo_url = format!("https://github.com/{}.git", plugin.current_entry.repo);
    let mut child = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            target_release_tag,
            "--single-branch",
            "--quiet",
            &repo_url,
        ])
        .arg(tmp_path)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| CloneError::GitStart(error.to_string()))?;

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let output = child
                    .wait_with_output()
                    .map_err(|error| CloneError::GitOutput(error.to_string()))?;
                if output.status.success() {
                    return Ok(());
                }

                return Err(CloneError::GitFailed(format_git_clone_error(
                    &output.stderr,
                )));
            }
            Ok(None) if started.elapsed() >= clone_timeout => {
                let _ = child.kill();
                let output = child.wait_with_output();
                let detail = output
                    .ok()
                    .map(|output| command_output_tail(&output.stderr))
                    .filter(|detail| !detail.is_empty());
                return Err(CloneError::GitTimeout {
                    seconds: clone_timeout.as_secs(),
                    detail,
                });
            }
            Ok(None) => thread::sleep(Duration::from_millis(250)),
            Err(error) => return Err(CloneError::GitWait(error.to_string())),
        }
    }
}

fn format_git_clone_error(stderr: &[u8]) -> String {
    let detail = command_output_tail(stderr);
    if detail.is_empty() {
        "git clone failed without stderr output".to_string()
    } else {
        detail
    }
}

fn command_output_tail(output: &[u8]) -> String {
    const MAX_ERROR_CHARS: usize = 2000;

    let text = String::from_utf8_lossy(output);
    let trimmed = text.trim();
    let char_count = trimmed.chars().count();
    if char_count <= MAX_ERROR_CHARS {
        return trimmed.to_string();
    }

    let tail = trimmed
        .chars()
        .rev()
        .take(MAX_ERROR_CHARS)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    format!("...{tail}")
}

#[cfg(test)]
mod tests {
    use super::{CloneError, CloneStatus};

    #[test]
    fn clone_error_status_keeps_failed_prefix_and_detail() {
        let error = CloneError::GitTimeout {
            seconds: 30,
            detail: Some("remote hung up".to_string()),
        };

        assert_eq!(
            error.as_state_value(),
            "failed:timed out after 30s: remote hung up"
        );
    }

    #[test]
    fn clone_status_values_match_persisted_labels() {
        assert_eq!(CloneStatus::Ok.as_state_value(), "ok");
        assert_eq!(
            CloneStatus::SkippedRemoved.as_state_value(),
            "skipped_removed"
        );
        assert_eq!(
            CloneStatus::VersionHistoryMissing.as_state_value(),
            "version_history_missing"
        );
    }
}
