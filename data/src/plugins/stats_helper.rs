use std::{
    error::Error,
    fmt,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use data_lib::{
    commit::Commit,
    common::VersionHistory,
    date::Date,
    plugin::{PluginData, PluginManifest},
    version::Version,
};
use hashbrown::HashMap;
use serde::Deserialize;

use crate::{
    constants::{OBSIDIAN_STATS_HELPER_REPO_PATH, STATS_HELPER_PLUGIN_DOWNLOADS_PATH},
    plugins::{PluginDownloadStat, PluginDownloadStats},
    progress::should_log_progress,
};

#[derive(Debug, Clone, Deserialize)]
pub struct HelperPluginData {
    pub id: String,
    pub repo: String,
    #[serde(default)]
    pub manifest: Option<PluginManifest>,
    #[serde(default)]
    pub releases: Vec<HelperRelease>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HelperRelease {
    pub tag: String,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(default)]
    #[serde(rename = "downloadCount")]
    pub download_count: Option<u32>,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct HelperDownloadSummary {
    plugins: HashMap<String, HelperDownloadSummaryEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct HelperDownloadSummaryEntry {
    downloads: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetRelease {
    pub tag: String,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetReleaseError {
    HelperPluginMissing,
    ManifestMissing,
    ManifestVersionMissing,
    ManifestVersionInvalid,
    ManifestVersionPrefixed,
    ReleaseForManifestVersionMissing,
}

impl TargetReleaseError {
    pub fn as_state_value(self) -> &'static str {
        match self {
            Self::HelperPluginMissing => "helper_plugin_missing",
            Self::ManifestMissing => "manifest_missing",
            Self::ManifestVersionMissing => "manifest_version_missing",
            Self::ManifestVersionInvalid => "manifest_version_invalid",
            Self::ManifestVersionPrefixed => "manifest_version_prefixed",
            Self::ReleaseForManifestVersionMissing => "release_for_manifest_version_missing",
        }
    }
}

impl fmt::Display for TargetReleaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_state_value())
    }
}

#[derive(Debug, Clone, Default)]
pub struct HelperPluginStore {
    plugins: HashMap<String, HelperPluginData>,
}

impl HelperPluginStore {
    pub fn read() -> Result<Self, Box<dyn Error>> {
        let plugin_dir = Path::new(OBSIDIAN_STATS_HELPER_REPO_PATH)
            .join("data")
            .join("plugins");
        let mut plugins = HashMap::new();

        for entry in std::fs::read_dir(&plugin_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            let plugin = read_helper_plugin(&path)?;
            plugins.insert(plugin.id.clone(), plugin);
        }

        Ok(Self { plugins })
    }

    pub fn get(&self, plugin_id: &str) -> Option<&HelperPluginData> {
        self.plugins.get(plugin_id)
    }

    pub fn target_release_for_plugin(
        &self,
        plugin: &PluginData,
    ) -> Result<TargetRelease, TargetReleaseError> {
        let helper_plugin = self
            .get(&plugin.id)
            .filter(|helper_plugin| helper_plugin.repo == plugin.current_entry.repo)
            .ok_or(TargetReleaseError::HelperPluginMissing)?;

        target_release(helper_plugin)
    }

    pub fn helper_manifest_for_plugin(&self, plugin: &PluginData) -> Option<PluginManifest> {
        self.get(&plugin.id)
            .filter(|helper_plugin| helper_plugin.repo == plugin.current_entry.repo)
            .and_then(|helper_plugin| helper_plugin.manifest.clone())
    }
}

pub fn build_version_history(helper_plugin: &HelperPluginData) -> Vec<VersionHistory> {
    let mut history = helper_plugin
        .releases
        .iter()
        .filter(|release| !release.draft)
        .filter_map(|release| {
            let version = release_version_from_tag(&release.tag)?;
            let published_at = release.published_at.as_deref()?;
            let date = date_from_iso_timestamp(published_at)?;
            let version_object = Version::parse(&version);

            Some(VersionHistory {
                version,
                version_object,
                initial_release_date: date,
                prerelease: release.prerelease,
                released_while_listed: true,
            })
        })
        .collect::<Vec<_>>();

    history.sort_by(|left, right| {
        left.initial_release_date
            .cmp(&right.initial_release_date)
            .then_with(|| left.version_object.cmp(&right.version_object))
            .then_with(|| left.version.cmp(&right.version))
    });
    history
}

pub fn load_helper_download_stat_history() -> Result<Vec<PluginDownloadStats>, Box<dyn Error>> {
    println!("Loading stats-helper plugin download history...");
    load_helper_download_summary_history()
}

fn load_helper_download_summary_history() -> Result<Vec<PluginDownloadStats>, Box<dyn Error>> {
    let repo_path = Path::new(OBSIDIAN_STATS_HELPER_REPO_PATH).canonicalize()?;
    let commits = get_helper_repo_changes_for_file(STATS_HELPER_PLUGIN_DOWNLOADS_PATH)?;

    println!(
        "Loading stats-helper download summaries from {} commit(s)...",
        commits.len()
    );
    let total_commits = commits.len();
    let mut history = Vec::with_capacity(commits.len());
    for (idx, commit) in commits.into_iter().enumerate() {
        if let Some(content) =
            read_file_at_commit(&repo_path, &commit, STATS_HELPER_PLUGIN_DOWNLOADS_PATH)?
        {
            append_latest_daily_download_summary(
                &mut history,
                helper_summary_to_download_stats(content, commit)?,
            );
        }

        let done = idx + 1;
        if should_log_progress(done, total_commits) {
            println!(
                "  Stats-helper download summary progress: {} / {}",
                done, total_commits
            );
        }
    }

    Ok(history)
}

fn helper_summary_to_download_stats(
    content: String,
    commit: Commit,
) -> Result<PluginDownloadStats, Box<dyn Error>> {
    let summary: HelperDownloadSummary = serde_json::from_str(&content)?;
    let entries = summary
        .plugins
        .into_iter()
        .map(|(id, entry)| {
            (
                id,
                PluginDownloadStat {
                    downloads: entry.downloads,
                },
            )
        })
        .collect();

    Ok(PluginDownloadStats { entries, commit })
}

fn append_latest_daily_download_summary(
    history: &mut Vec<PluginDownloadStats>,
    stats: PluginDownloadStats,
) {
    if history
        .last()
        .is_some_and(|previous| previous.commit.date == stats.commit.date)
    {
        *history.last_mut().expect("history has last entry") = stats;
    } else {
        history.push(stats);
    }
}

fn get_helper_repo_changes_for_file(file_path: &str) -> Result<Vec<Commit>, Box<dyn Error>> {
    let repo_path = Path::new(OBSIDIAN_STATS_HELPER_REPO_PATH).canonicalize()?;
    let output = std::process::Command::new("git")
        .args([
            "--no-pager",
            "log",
            "--date-order",
            "--reverse",
            "--format=\"%ad %H\"",
            "--date=iso-strict",
            "--diff-filter=AM",
            "--",
            file_path,
        ])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "git log failed for stats-helper {file_path}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }

    Ok(Commit::from_git_log(
        String::from_utf8_lossy(&output.stdout).to_string(),
    ))
}

fn read_file_at_commit(
    repo_path: &Path,
    commit: &Commit,
    file_path: &str,
) -> Result<Option<String>, Box<dyn Error>> {
    let output = std::process::Command::new("git")
        .args(["cat-file", "-p", &format!("{}:{file_path}", commit.hash)])
        .current_dir(repo_path)
        .output()?;

    if output.status.success() {
        return Ok(Some(String::from_utf8_lossy(&output.stdout).to_string()));
    }

    Ok(None)
}

pub fn target_release(
    helper_plugin: &HelperPluginData,
) -> Result<TargetRelease, TargetReleaseError> {
    let manifest = helper_plugin
        .manifest
        .as_ref()
        .ok_or(TargetReleaseError::ManifestMissing)?;
    let version = manifest
        .version
        .as_deref()
        .map(str::trim)
        .filter(|version| !version.is_empty())
        .ok_or(TargetReleaseError::ManifestVersionMissing)?;

    if has_version_prefix(version) {
        return Err(TargetReleaseError::ManifestVersionPrefixed);
    }

    if !is_valid_bare_manifest_version(version) {
        return Err(TargetReleaseError::ManifestVersionInvalid);
    }

    let prefixed = format!("v{version}");
    let uppercase_prefixed = format!("V{version}");
    let release = helper_plugin
        .releases
        .iter()
        .filter(|release| !release.draft)
        .find(|release| {
            release.tag == version || release.tag == prefixed || release.tag == uppercase_prefixed
        })
        .ok_or(TargetReleaseError::ReleaseForManifestVersionMissing)?;

    Ok(TargetRelease {
        tag: release.tag.clone(),
        version: version.to_string(),
    })
}

fn read_helper_plugin(path: &PathBuf) -> Result<HelperPluginData, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

fn release_version_from_tag(tag: &str) -> Option<String> {
    let version = tag
        .strip_prefix('v')
        .or_else(|| tag.strip_prefix('V'))
        .unwrap_or(tag);
    if Version::validate(version) {
        Some(version.to_string())
    } else {
        None
    }
}

fn has_version_prefix(version: &str) -> bool {
    version.starts_with('v') || version.starts_with('V')
}

fn is_valid_bare_manifest_version(version: &str) -> bool {
    let core = version.split_once('-').map_or(version, |(core, _)| core);
    core.split('.').count() == 3 && Version::validate(version)
}

fn date_from_iso_timestamp(timestamp: &str) -> Option<Date> {
    timestamp.get(..10).and_then(Date::from_string)
}

#[cfg(test)]
mod tests {
    use super::{
        HelperPluginData, HelperRelease, TargetReleaseError, append_latest_daily_download_summary,
        build_version_history, target_release,
    };
    use crate::plugins::PluginDownloadStats;
    use data_lib::plugin::PluginManifest;
    use data_lib::{commit::Commit, date::Date};
    use hashbrown::HashMap;

    fn helper_plugin(
        manifest_version: Option<&str>,
        releases: Vec<HelperRelease>,
    ) -> HelperPluginData {
        HelperPluginData {
            id: "plugin".to_string(),
            repo: "owner/repo".to_string(),
            manifest: Some(PluginManifest {
                version: manifest_version.map(str::to_string),
                ..PluginManifest::default()
            }),
            releases,
        }
    }

    fn release(tag: &str, prerelease: bool, draft: bool) -> HelperRelease {
        HelperRelease {
            tag: tag.to_string(),
            published_at: Some("2026-01-02T03:04:05Z".to_string()),
            download_count: Some(0),
            prerelease,
            draft,
        }
    }

    #[test]
    fn version_history_includes_prereleases_and_ignores_drafts() {
        let plugin = helper_plugin(
            Some("1.1.0"),
            vec![
                release("1.0.0", false, false),
                release("v1.1.0-beta", true, false),
                release("1.2.0", false, true),
            ],
        );

        let history = build_version_history(&plugin);

        assert_eq!(history.len(), 2);
        assert_eq!(history[0].version, "1.0.0");
        assert!(!history[0].prerelease);
        assert_eq!(history[1].version, "1.1.0-beta");
        assert!(history[1].prerelease);
    }

    #[test]
    fn append_latest_daily_download_summary_keeps_latest_snapshot_per_day() {
        let mut history = vec![PluginDownloadStats {
            commit: Commit {
                date: Date::new(2026, 6, 26),
                hash: "morning".to_string(),
            },
            entries: HashMap::new(),
        }];

        append_latest_daily_download_summary(
            &mut history,
            PluginDownloadStats {
                commit: Commit {
                    date: Date::new(2026, 6, 26),
                    hash: "evening".to_string(),
                },
                entries: HashMap::new(),
            },
        );
        append_latest_daily_download_summary(
            &mut history,
            PluginDownloadStats {
                commit: Commit {
                    date: Date::new(2026, 6, 27),
                    hash: "next-day".to_string(),
                },
                entries: HashMap::new(),
            },
        );

        assert_eq!(history.len(), 2);
        assert_eq!(history[0].commit.hash, "evening");
        assert_eq!(history[1].commit.hash, "next-day");
    }

    #[test]
    fn target_release_allows_prefixed_release_tag_for_bare_manifest_version() {
        let plugin = helper_plugin(Some("1.2.3"), vec![release("v1.2.3", false, false)]);

        let target = target_release(&plugin).unwrap();

        assert_eq!(target.tag, "v1.2.3");
        assert_eq!(target.version, "1.2.3");
    }

    #[test]
    fn target_release_rejects_prefixed_manifest_version() {
        let plugin = helper_plugin(Some("v1.2.3"), vec![release("v1.2.3", false, false)]);

        assert_eq!(
            target_release(&plugin).unwrap_err(),
            TargetReleaseError::ManifestVersionPrefixed
        );
    }

    #[test]
    fn target_release_requires_three_part_manifest_version() {
        let plugin = helper_plugin(Some("1.2"), vec![release("1.2", false, false)]);

        assert_eq!(
            target_release(&plugin).unwrap_err(),
            TargetReleaseError::ManifestVersionInvalid
        );
    }

    #[test]
    fn target_release_requires_matching_release_tag() {
        let plugin = helper_plugin(Some("1.2.3"), vec![release("1.2.2", false, false)]);

        assert_eq!(
            target_release(&plugin).unwrap_err(),
            TargetReleaseError::ReleaseForManifestVersionMissing
        );
    }
}
