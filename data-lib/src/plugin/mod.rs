use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::{commit::{Commit, StringCommit}, common::{DownloadHistory, EntryChange, VersionHistory}, input_data::ObsCommunityPlugin, plugin::{bundlers::Bundler, packages::PackageManager, testing::TestingFramework}};

pub mod bundlers;
pub mod packages;
pub mod testing;
pub mod analysis;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
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
    pub package_json_license: String,
    pub license_file: String,
    pub manifest: PluginManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExtraData {
    pub id: String,
    pub repo: Result<PluginRepoData, String>,
    pub removal_reason: Option<String>,
    pub deprecated_versions: Vec<String>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct DownloadDataPoint {
    pub date: String,
    pub downloads: Option<u32>,
    pub delta: Option<u32>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct VersionDataPoint {
    pub version: String,
    pub date: String,
    pub deprecated: bool,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct EntryChangeDataPoint {
    pub property: String,
    pub commit: StringCommit,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct IndividualDownloadDataPoint {
    pub id: String,
    pub name: String,
    pub date: String,
    pub downloads: u32,
    pub version_count: u32,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginOverviewDataPoint {
    pub id: String,
    pub name: String,
    pub author: String,
    pub repo: String,
    pub repo_url: String,
    pub added_commit: StringCommit,
    pub removed_commit: Option<StringCommit>,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginYearlyDataPoint {
    pub id: String,
    pub name: String,
    pub downloads_new: u32,
    pub downloads_start: u32,
    pub data: Vec<DownloadDataPoint>,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginCountMonthlyDataPoint {
    pub date: String,
    pub total: u32,
    pub total_with_removed: u32,
    pub new: u32,
    pub new_removed: u32,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginRemovedByReleaseDataPoint {
    pub date: String,
    pub percentage: f64,
}