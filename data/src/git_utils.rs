use std::{path::Path, process::Command};

use data_lib::commit::Commit;

use crate::constants::OBS_RELEASES_REPO_PATH;

pub fn get_obs_repo_changes() -> Vec<Commit> {
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
