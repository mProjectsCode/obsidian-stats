use std::{path::Path, process::Command};

use data_lib::commit::Commit;

use crate::constants::OBS_RELEASES_REPO_PATH;

pub fn get_obs_repo_changes() -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
    get_obs_repo_changes_for_path(None)
}

pub fn get_obs_repo_changes_for_file(
    file_path: &str,
) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
    get_obs_repo_changes_for_path(Some(file_path))
}

fn get_obs_repo_changes_for_path(
    file_path: Option<&str>,
) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
    let repo_path = Path::new(OBS_RELEASES_REPO_PATH).canonicalize()?;
    let mut args = vec![
        "--no-pager",
        "log",
        "--date-order",
        "--reverse",
        "--format=\"%ad %H\"",
        "--date=iso-strict",
    ];

    match file_path {
        Some(file_path) => {
            args.push("--diff-filter=AM");
            args.extend(["--", file_path]);
        }
        None => {
            args.push("--diff-filter=M");
            args.push("--grep=stats\\|chore");
        }
    }

    let git_output = Command::new("git")
        .args(args)
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
