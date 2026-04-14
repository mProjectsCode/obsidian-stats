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
    pub removed_commit: Option<Commit>,
    pub initial_entry: ObsCommunityPlugin,
    pub current_entry: ObsCommunityPlugin,
    pub change_history: Vec<EntryChange>,
    pub download_history: DownloadHistory,
    pub download_count: u32,
    pub version_history: Vec<VersionHistory>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[tsify(into_wasm_abi)]
pub enum FundingUrl {
    String(String),
    Object(HashMap<String, String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub author: Option<String>,
    #[serde(rename = "minAppVersion")]
    pub min_app_version: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "authorUrl")]
    pub author_url: Option<String>,
    #[serde(rename = "fundingUrl")]
    pub funding_url: Option<FundingUrl>,

    pub description: Option<String>,
    pub id: Option<String>,
    #[serde(rename = "isDesktopOnly")]
    pub is_desktop_only: Option<bool>,

    // Non-standard fields
    #[serde(rename = "helpUrl")]
    pub help_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRepoData {
    pub uses_typescript: bool,
    pub has_package_json: bool,
    pub package_managers: Vec<PackageManager>,
    pub testing_frameworks: Vec<TestingFramework>,
    pub bundlers: Vec<Bundler>,
    pub dependencies: Vec<String>,
    pub dev_dependencies: Vec<String>,
    pub has_test_files: bool,
    pub has_beta_manifest: bool,
    pub file_type_counts: HashMap<String, usize>,
    /// The license identifier from the package.json file.
    pub package_json_license: LicenseInfo,
    /// The license identifier from the LICENSE file in the repository.
    pub file_license: LicenseInfo,
    pub manifest: Option<PluginManifest>,
    pub lines_of_code: HashMap<String, usize>,
    pub has_i18n_dependencies: bool,
    pub has_i18n_files: bool,
    pub latest_release_main_js_size_bytes: Option<u64>,
    pub estimated_target_es_version: Option<String>,
    pub main_js_is_probably_minified: Option<bool>,
    pub main_js_minification_score: Option<f32>,
    pub main_js_includes_sourcemap_comment: Option<bool>,
    pub main_js_large_base64_blob_count: Option<u32>,
    pub main_js_largest_base64_blob_length: Option<u32>,
    pub main_js_worker_usage_count: Option<u32>,
    pub main_js_webassembly_usage_count: Option<u32>,
    pub latest_release_tag: Option<String>,
    pub latest_release_published_at: Option<String>,
    pub latest_release_fetch_status: Option<String>,
    #[serde(default)]
    pub analysis_errors: Vec<PluginRepoAnalysisError>,
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
    pub removal_reason: Option<String>,
    pub deprecated_versions: Vec<String>,
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
