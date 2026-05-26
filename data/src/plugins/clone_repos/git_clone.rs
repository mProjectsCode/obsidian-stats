use std::{
    fmt,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use data_lib::plugin::PluginData;

use crate::{
    constants::PLUGIN_REPO_PATH,
    security::{github_repo_url, validated_plugin_path},
    state::now_unix_seconds,
};

pub(super) enum CloneResult {
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

#[derive(Debug, Clone)]
pub(super) enum CloneError {
    InvalidPluginId(String),
    InvalidRepoSlug(String),
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
    pub(super) fn as_state_value(&self) -> String {
        format!("failed:{self}")
    }
}

impl fmt::Display for CloneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPluginId(error) => write!(f, "{error}"),
            Self::InvalidRepoSlug(error) => write!(f, "{error}"),
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

pub(super) fn clone_repo_preserving_previous(
    plugin: &PluginData,
    target_release_tag: &str,
    clone_timeout: Duration,
) -> CloneResult {
    let target_path = match validated_plugin_path(Path::new(PLUGIN_REPO_PATH), &plugin.id) {
        Ok(path) => path,
        Err(error) => {
            return CloneResult::Failed {
                id: plugin.id.clone(),
                repo: plugin.current_entry.repo.clone(),
                target_release_tag: target_release_tag.to_string(),
                error: CloneError::InvalidPluginId(error),
            };
        }
    };
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
    let repo_url =
        github_repo_url(&plugin.current_entry.repo).map_err(CloneError::InvalidRepoSlug)?;
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
    use super::CloneError;

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
}
