use std::{
    error::Error,
    fmt,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use data_lib::{
    common::VersionHistory,
    date::Date,
    plugin::{PluginData, PluginManifest},
    version::Version,
};
use hashbrown::HashMap;
use serde::Deserialize;

use crate::constants::OBSIDIAN_STATS_HELPER_REPO_PATH;

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
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
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
        HelperPluginData, HelperRelease, TargetReleaseError, build_version_history, target_release,
    };
    use data_lib::plugin::PluginManifest;

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
