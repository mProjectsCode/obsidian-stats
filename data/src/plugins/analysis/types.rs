use data_lib::plugin::{
    LicenseInfo, PluginManifest, PluginRepoAnalysisError, bundlers::Bundler,
    packages::PackageManager, testing::TestingFramework,
};
use hashbrown::HashMap;
use std::collections::BTreeMap;

use super::mainjs::api_classifier::ApiClassificationResult;

#[derive(Debug, Default)]
pub(super) struct AnalysisResult {
    pub(super) mainjs: MainJsResult,
}

#[derive(Debug)]
pub(super) struct RepoResult {
    pub(super) uses_typescript: bool,
    pub(super) has_package_json: bool,
    pub(super) package_managers: Vec<PackageManager>,
    pub(super) testing_frameworks: Vec<TestingFramework>,
    pub(super) bundlers: Vec<Bundler>,
    pub(super) dependencies: Vec<String>,
    pub(super) dev_dependencies: Vec<String>,
    pub(super) has_test_files: bool,
    pub(super) has_beta_manifest: bool,
    pub(super) file_type_counts: HashMap<String, usize>,
    pub(super) package_json_license: LicenseInfo,
    pub(super) file_license: LicenseInfo,
    pub(super) manifest: Option<PluginManifest>,
    pub(super) lines_of_code: HashMap<String, usize>,
    pub(super) has_i18n_dependencies: bool,
    pub(super) has_i18n_files: bool,
    pub(super) analysis_errors: Vec<PluginRepoAnalysisError>,
}

#[derive(Debug, Default)]
pub(super) struct MainJsResult {
    pub(super) parse_succeeded: Option<bool>,
    pub(super) tolerant_parse_required: Option<bool>,
    pub(super) estimated_target_es_version: Option<String>,
    pub(super) is_probably_minified: Option<bool>,
    pub(super) minification_score: Option<f32>,
    pub(super) includes_sourcemap_comment: Option<bool>,
    pub(super) includes_inline_sourcemap: Option<bool>,
    pub(super) large_base64_blob_count: Option<u32>,
    pub(super) largest_base64_blob_length: Option<u32>,
    pub(super) embedded_blob_type_counts: BTreeMap<String, u32>,
    pub(super) worker_usage_count: Option<u32>,
    pub(super) webassembly_usage_count: Option<u32>,
    pub(super) dynamic_import_usage_count: Option<u32>,
    pub(super) bundler_fingerprints: Vec<String>,
    pub(super) module_system_fingerprints: Vec<String>,
    pub(super) size_bucket: Option<String>,
    pub(super) line_count_bucket: Option<String>,
    pub(super) uses_optional_chaining: Option<bool>,
    pub(super) uses_nullish_coalescing: Option<bool>,
    pub(super) uses_private_fields: Option<bool>,
    pub(super) uses_top_level_await: Option<bool>,
    pub(super) known_api_host_counts: BTreeMap<String, u32>,
    pub(super) embedded_dependency_name_counts: BTreeMap<String, u32>,
    pub(super) license_banner_count: Option<u32>,
    pub(super) credential_literal_count: Option<u32>,
    pub(super) api_usage: ApiClassificationResult,
}
