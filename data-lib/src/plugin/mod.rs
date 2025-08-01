use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
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
    pub author: String,
    #[serde(rename = "minAppVersion")]
    pub min_app_version: String,
    pub name: String,
    pub version: String,
    #[serde(rename = "authorUrl")]
    pub author_url: Option<String>,
    #[serde(rename = "fundingUrl")]
    pub funding_url: Option<FundingUrl>,

    pub description: String,
    pub id: String,
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
    pub manifest: PluginManifest,
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
