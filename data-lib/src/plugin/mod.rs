use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{commit::Commit, common::{DownloadHistory, EntryChange, VersionHistory}, input_data::ObsCommunityPlugin, plugin::{bundlers::Bundler, packages::PackageManager, testing::TestingFramework}};

pub mod bundlers;
pub mod packages;
pub mod testing;
pub mod analysis;

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
pub enum PluginWarningSeverity {
    CAUTION,
    DANGER,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "id")]
pub enum PluginWarning {
    Inactivity12Months(PluginWarningInactivity12Months),
    Inactivity24Months(PluginWarningInactivity24Months),
    MismatchedManifestData(PluginWarningMismatchedManifestData),
    Unlicensed(PluginWarningUnlicensed),
    NoLicense(PluginWarningNoLicense),
    MismatchedLicense(PluginWarningMismatchedLicense),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginWarningInactivity12Months {
    pub severity: PluginWarningSeverity,
    pub last_release_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginWarningInactivity24Months {
    pub severity: PluginWarningSeverity,
    pub last_release_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginWarningMismatchedManifestDataField {
    pub field: String,
    pub manifest_value: String,
    pub community_list_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginWarningMismatchedManifestData {
    pub severity: PluginWarningSeverity,
    pub data: Vec<PluginWarningMismatchedManifestDataField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginWarningUnlicensed {
    pub severity: PluginWarningSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginWarningNoLicense {
    pub severity: PluginWarningSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginWarningMismatchedLicense {
    pub severity: PluginWarningSeverity,
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
pub struct PluginRepoExtractedData {
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
pub struct PluginRepoData {
    pub id: String,
    pub repo: Result<PluginRepoExtractedData, String>,
    pub warnings: Vec<PluginWarning>,
    pub removal_reason: Option<String>,
    pub deprecated_versions: Vec<String>,
}