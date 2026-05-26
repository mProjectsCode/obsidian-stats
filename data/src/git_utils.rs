use std::{path::Path, process::Command};

use data_lib::commit::Commit;

use crate::constants::OBS_RELEASES_REPO_PATH;

pub fn get_obs_repo_changes() -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
    let repo_path = Path::new(OBS_RELEASES_REPO_PATH).canonicalize()?;
    let git_output = Command::new("git")
        .args([
            "--no-pager",
            "log",
            "--diff-filter=M",
            "--date-order",
            "--reverse",
            "--format=\"%ad %H\"",
            "--date=iso-strict",
            "--grep=stats\\|chore",
        ])
        .current_dir(repo_path)
        .output()?;

    if !git_output.status.success() {
        return Err(format!(
            "git log failed: {}",
            String::from_utf8_lossy(&git_output.stderr).trim()
        )
        .into());
    }

    Ok(Commit::from_git_log(
        String::from_utf8_lossy(&git_output.stdout).to_string(),
    ))
}
