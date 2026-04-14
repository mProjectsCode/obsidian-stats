use data_lib::plugin::{
    LicenseInfo, PluginManifest, PluginRepoAnalysisError, bundlers::Bundler,
    packages::PackageManager, testing::TestingFramework,
};
use hashbrown::HashMap;

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
    pub(super) estimated_target_es_version: Option<String>,
    pub(super) is_probably_minified: Option<bool>,
    pub(super) minification_score: Option<f32>,
    pub(super) includes_sourcemap_comment: Option<bool>,
    pub(super) large_base64_blob_count: Option<u32>,
    pub(super) largest_base64_blob_length: Option<u32>,
    pub(super) worker_usage_count: Option<u32>,
    pub(super) webassembly_usage_count: Option<u32>,
}
