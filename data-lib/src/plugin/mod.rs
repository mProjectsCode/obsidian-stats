use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tsify::Tsify;

use crate::{
    commit::Commit,
    common::{DownloadHistory, EntryChange, NamedDataPoint, VersionHistory},
    input_data::ObsCommunityPlugin,
    license::LicenseDescriptionNested,
    plugin::{bundlers::Bundler, packages::PackageManager, testing::TestingFramework},
};

pub mod bundlers;
pub mod data_array;
pub mod full;
pub mod milestones;
pub mod packages;
pub mod testing;
pub mod warnings;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginData {
    pub id: String,
    pub added_commit: Commit,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed_commit: Option<Commit>,
    pub initial_entry: ObsCommunityPlugin,
    pub current_entry: ObsCommunityPlugin,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub change_history: Vec<EntryChange>,
    #[serde(default, skip_serializing_if = "download_history_is_empty")]
    pub download_history: DownloadHistory,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub download_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub version_history: Vec<VersionHistory>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[tsify(into_wasm_abi)]
pub enum FundingUrl {
    String(String),
    Object(HashMap<String, String>),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginManifest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(rename = "minAppVersion")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_app_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(rename = "authorUrl")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_url: Option<String>,
    #[serde(rename = "fundingUrl")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funding_url: Option<FundingUrl>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "isDesktopOnly")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_desktop_only: Option<bool>,

    // Non-standard fields
    #[serde(rename = "helpUrl")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRepoData {
    #[serde(default, skip_serializing_if = "is_false")]
    pub uses_typescript: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_package_json: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub package_managers: Vec<PackageManager>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub testing_frameworks: Vec<TestingFramework>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bundlers: Vec<Bundler>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dev_dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_test_files: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_beta_manifest: bool,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub file_type_counts: HashMap<String, usize>,
    /// The license identifier from the package.json file.
    #[serde(default, skip_serializing_if = "is_not_found")]
    pub package_json_license: LicenseInfo,
    /// The license identifier from the LICENSE file in the repository.
    #[serde(default, skip_serializing_if = "is_not_found")]
    pub file_license: LicenseInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<PluginManifest>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub lines_of_code: HashMap<String, usize>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_i18n_dependencies: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_i18n_files: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_release_main_js_size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_target_es_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_js_is_probably_minified: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_js_minification_score: Option<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub main_js_findings: Vec<MainJsFinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub main_js_disclosures: Vec<MainJsDisclosure>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub main_js_diagnostics: Vec<MainJsDiagnostic>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_release_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_release_published_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_release_fetch_status: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub analysis_errors: Vec<PluginRepoAnalysisError>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct MainJsFinding {
    pub rule_id: String,
    pub description: String,
    pub message: String,
    pub severity: String,
    pub confidence: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<MainJsEvidence>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct MainJsDisclosure {
    pub id: String,
    pub from_rule_id: String,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct MainJsEvidence {
    pub message: String,
    #[serde(default = "default_evidence_count")]
    pub count: u32,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct MainJsLocation {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct MainJsDiagnostic {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<MainJsLocation>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginRepoAnalysisError {
    #[serde(rename = "manifest_read_error")]
    ManifestRead,
    #[serde(rename = "manifest_parse_error")]
    ManifestParse,
    #[serde(rename = "package_json_read_error")]
    PackageJsonRead,
    #[serde(rename = "package_json_parse_error")]
    PackageJsonParse,
    #[serde(rename = "repository_missing")]
    RepositoryMissing,
    #[serde(rename = "repo_missing")]
    RepoMissing,
    #[serde(rename = "repo_analysis_error")]
    RepoAnalysis,
    #[serde(rename = "repository_scan_error")]
    RepositoryScan,
}

#[derive(Debug, Error)]
pub enum PluginRepoAnalysisDetailError {
    #[error("repository_missing: plugin {plugin_id}: {path}")]
    RepositoryMissing { plugin_id: String, path: PathBuf },

    #[error("manifest_read_error: plugin {plugin_id}: {source}")]
    ManifestRead {
        plugin_id: String,
        #[source]
        source: std::io::Error,
    },

    #[error("manifest_parse_error: plugin {plugin_id}: {source}")]
    ManifestParse {
        plugin_id: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("package_json_read_error: plugin {plugin_id}: {source}")]
    PackageJsonRead {
        plugin_id: String,
        #[source]
        source: std::io::Error,
    },

    #[error("package_json_parse_error: plugin {plugin_id}: {source}")]
    PackageJsonParse {
        plugin_id: String,
        #[source]
        source: serde_json::Error,
    },
}

impl PluginRepoAnalysisDetailError {
    pub fn code(&self) -> PluginRepoAnalysisError {
        match self {
            Self::RepositoryMissing { .. } => PluginRepoAnalysisError::RepositoryMissing,
            Self::ManifestRead { .. } => PluginRepoAnalysisError::ManifestRead,
            Self::ManifestParse { .. } => PluginRepoAnalysisError::ManifestParse,
            Self::PackageJsonRead { .. } => PluginRepoAnalysisError::PackageJsonRead,
            Self::PackageJsonParse { .. } => PluginRepoAnalysisError::PackageJsonParse,
        }
    }
}

impl PluginRepoAnalysisError {
    pub fn as_label(self) -> &'static str {
        match self {
            Self::ManifestRead => "manifest_read_error",
            Self::ManifestParse => "manifest_parse_error",
            Self::PackageJsonRead => "package_json_read_error",
            Self::PackageJsonParse => "package_json_parse_error",
            Self::RepositoryMissing => "repository_missing",
            Self::RepoMissing => "repo_missing",
            Self::RepoAnalysis => "repo_analysis_error",
            Self::RepositoryScan => "repository_scan_error",
        }
    }

    pub fn from_raw(error: &str) -> Self {
        if let Some((prefix, _)) = error.split_once(':') {
            return Self::from_raw(prefix);
        }

        match error {
            "manifest_read_error" => Self::ManifestRead,
            "manifest_parse_error" => Self::ManifestParse,
            "package_json_read_error" => Self::PackageJsonRead,
            "package_json_parse_error" => Self::PackageJsonParse,
            "repository_missing" => Self::RepositoryMissing,
            "repo_missing" => Self::RepoMissing,
            "repository_scan_error" => Self::RepositoryScan,
            _ => {
                if error.contains("does not exist") {
                    Self::RepoMissing
                } else if error.contains("Failed to read manifest") {
                    Self::ManifestRead
                } else if error.contains("Failed to parse manifest") {
                    Self::ManifestParse
                } else if error.contains("Failed to read package.json") {
                    Self::PackageJsonRead
                } else if error.contains("Failed to parse package.json") {
                    Self::PackageJsonParse
                } else {
                    Self::RepoAnalysis
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExtraData {
    pub id: String,
    pub repo: Result<PluginRepoData, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removal_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deprecated_versions: Vec<String>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

fn download_history_is_empty(value: &DownloadHistory) -> bool {
    value.0.is_empty()
}

fn is_not_found(value: &LicenseInfo) -> bool {
    matches!(value, LicenseInfo::NotFound)
}

fn default_evidence_count() -> u32 {
    1
}

#[cfg(test)]
mod serialization_tests {
    use super::*;
    use crate::{
        commit::Commit,
        date::Date,
        input_data::ObsCommunityPlugin,
        release::{GithubAssetInfo, GithubReleaseInfo, ObsidianPlatform, ObsidianReleaseInfo},
        theme::ThemeData,
        version::Version,
    };

    fn commit() -> Commit {
        Commit {
            date: Date::new(2025, 1, 1),
            hash: "fixture".to_string(),
        }
    }

    fn plugin_data() -> PluginData {
        let entry = ObsCommunityPlugin {
            id: "fixture".to_string(),
            name: "Fixture".to_string(),
            author: "Author".to_string(),
            description: "Description".to_string(),
            repo: "author/fixture".to_string(),
        };
        PluginData {
            id: "fixture".to_string(),
            added_commit: commit(),
            removed_commit: None,
            initial_entry: entry.clone(),
            current_entry: entry,
            change_history: Vec::new(),
            download_history: DownloadHistory::default(),
            download_count: 0,
            version_history: Vec::new(),
        }
    }

    fn repo_data() -> PluginRepoData {
        PluginRepoData {
            uses_typescript: false,
            has_package_json: false,
            package_managers: Vec::new(),
            testing_frameworks: Vec::new(),
            bundlers: Vec::new(),
            dependencies: Vec::new(),
            dev_dependencies: Vec::new(),
            has_test_files: false,
            has_beta_manifest: false,
            file_type_counts: HashMap::new(),
            package_json_license: LicenseInfo::NotFound,
            file_license: LicenseInfo::NotFound,
            manifest: None,
            lines_of_code: HashMap::new(),
            has_i18n_dependencies: false,
            has_i18n_files: false,
            latest_release_main_js_size_bytes: None,
            estimated_target_es_version: None,
            main_js_is_probably_minified: None,
            main_js_minification_score: None,
            main_js_findings: Vec::new(),
            main_js_disclosures: Vec::new(),
            main_js_diagnostics: Vec::new(),
            latest_release_tag: None,
            latest_release_published_at: None,
            latest_release_fetch_status: None,
            analysis_errors: Vec::new(),
        }
    }

    fn assert_round_trip<T>(value: &T) -> usize
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        let bytes = serde_json::to_vec(value).unwrap();
        let decoded: T = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            serde_json::to_value(value).unwrap(),
            serde_json::to_value(decoded).unwrap()
        );
        bytes.len()
    }

    #[test]
    fn compact_generated_records_round_trip_and_omit_empty_fields() {
        let plugin_bytes = assert_round_trip(&plugin_data());
        let extra_bytes = assert_round_trip(&PluginExtraData {
            id: "fixture".to_string(),
            repo: Ok(repo_data()),
            removal_reason: None,
            deprecated_versions: Vec::new(),
        });
        let theme_bytes = assert_round_trip(&ThemeData {
            id: "fixture".to_string(),
            name: "Fixture".to_string(),
            added_commit: commit(),
            removed_commit: None,
            initial_entry: crate::input_data::ObsCommunityTheme {
                name: "Fixture".to_string(),
                author: "Author".to_string(),
                repo: "author/fixture".to_string(),
                screenshot: "screenshot.png".to_string(),
                modes: Vec::new(),
                legacy: false,
            },
            current_entry: crate::input_data::ObsCommunityTheme {
                name: "Fixture".to_string(),
                author: "Author".to_string(),
                repo: "author/fixture".to_string(),
                screenshot: "screenshot.png".to_string(),
                modes: Vec::new(),
                legacy: false,
            },
            change_history: Vec::new(),
        });
        let release_bytes = assert_round_trip(&GithubReleaseInfo {
            version: Version::new(1, 0, 0, None),
            date: Date::new(2025, 1, 1),
            time: "00:00:00".to_string(),
            assets: vec![GithubAssetInfo {
                name: "fixture".to_string(),
                downloads: HashMap::new(),
                size: 0,
            }],
        });
        let _ = assert_round_trip(&ObsidianReleaseInfo {
            version: Version::new(1, 0, 0, None),
            platform: ObsidianPlatform::Desktop,
            insider: false,
            date: Date::new(2025, 1, 1),
            info: String::new(),
            major_release: false,
        });

        let json = serde_json::to_string(&repo_data()).unwrap();
        assert!(!json.contains("main_js_findings"));
        assert!(!json.contains("package_managers"));
        assert!(plugin_bytes > 0 && extra_bytes > 0 && theme_bytes > 0 && release_bytes > 0);
    }

    #[test]
    fn legacy_evidence_defaults_to_one_occurrence() {
        let evidence: MainJsEvidence =
            serde_json::from_str(r#"{"trace_index":0,"role":"call","message":"call of fetch"}"#)
                .unwrap();
        assert_eq!(evidence.message, "call of fetch");
        assert_eq!(evidence.count, 1);
    }
}

#[derive(Tsify, Debug, Clone, Serialize, Default)]
#[tsify(into_wasm_abi)]
/// All data is in percentages.
pub struct PluginRepoDataPoints {
    bundlers: Vec<NamedDataPoint>,
    no_bundlers: f64,
    package_managers: Vec<NamedDataPoint>,
    no_package_managers: f64,
    testing_frameworks: Vec<NamedDataPoint>,
    no_testing_frameworks: f64,
    dependencies: Vec<NamedDataPoint>,
    beta_manifest: f64,
    typescript: f64,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginLicenseDataPoints {
    licenses: Vec<NamedDataPoint>,
    permissions: Vec<NamedDataPoint>,
    conditions: Vec<NamedDataPoint>,
    limitations: Vec<NamedDataPoint>,
    descriptions: LicenseDescriptionNested,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LicenseInfo {
    Known(String),
    ExplicitlyUnlicensed,
    Unrecognized,
    NotFound,
}

impl Default for LicenseInfo {
    fn default() -> Self {
        Self::NotFound
    }
}

impl LicenseInfo {
    pub fn to_fancy_string(&self) -> String {
        match self {
            LicenseInfo::Known(name) => name.clone(),
            LicenseInfo::Unrecognized => "Unrecognized".to_string(),
            LicenseInfo::NotFound => "Not Found".to_string(),
            LicenseInfo::ExplicitlyUnlicensed => "Explicitly Unlicensed".to_string(),
        }
    }

    pub fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (LicenseInfo::Known(name1), LicenseInfo::Known(name2)) => name1 == name2,
            _ => true, // Unrecognized and NotFound match anything
        }
    }

    pub fn matches_identifier(&self, other: &str) -> bool {
        match self {
            LicenseInfo::Known(name1) => name1 == other,
            _ => false,
        }
    }
}

impl From<Option<LicenseInfo>> for LicenseInfo {
    fn from(value: Option<LicenseInfo>) -> Self {
        value.unwrap_or(LicenseInfo::NotFound)
    }
}

impl From<Option<&LicenseInfo>> for LicenseInfo {
    fn from(value: Option<&LicenseInfo>) -> Self {
        value.cloned().unwrap_or(LicenseInfo::NotFound)
    }
}
