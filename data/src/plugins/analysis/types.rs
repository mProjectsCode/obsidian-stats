use data_lib::plugin::{
    LicenseInfo, PluginManifest, PluginRepoAnalysisError, PluginRepoData, bundlers::Bundler,
    packages::PackageManager, testing::TestingFramework,
};
use hashbrown::HashMap;
use std::collections::BTreeMap;

use super::mainjs::api_classifier::ApiClassificationResult;

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

impl RepoResult {
    pub(super) fn into_plugin_repo_data(self) -> PluginRepoData {
        PluginRepoData {
            uses_typescript: self.uses_typescript,
            has_package_json: self.has_package_json,
            package_managers: self.package_managers,
            testing_frameworks: self.testing_frameworks,
            bundlers: self.bundlers,
            dependencies: self.dependencies,
            dev_dependencies: self.dev_dependencies,
            has_test_files: self.has_test_files,
            has_beta_manifest: self.has_beta_manifest,
            file_type_counts: self.file_type_counts,
            package_json_license: self.package_json_license,
            file_license: self.file_license,
            manifest: self.manifest,
            lines_of_code: self.lines_of_code,
            has_i18n_dependencies: self.has_i18n_dependencies,
            has_i18n_files: self.has_i18n_files,
            latest_release_main_js_size_bytes: None,
            main_js_parse_succeeded: None,
            main_js_tolerant_parse_required: None,
            estimated_target_es_version: None,
            main_js_is_probably_minified: None,
            main_js_minification_score: None,
            main_js_dynamic_import_usage_count: None,
            main_js_bundler_fingerprints: Vec::new(),
            main_js_module_system_fingerprints: Vec::new(),
            main_js_size_bucket: None,
            main_js_line_count_bucket: None,
            main_js_uses_optional_chaining: None,
            main_js_uses_nullish_coalescing: None,
            main_js_uses_private_fields: None,
            main_js_uses_top_level_await: None,
            main_js_known_api_host_counts: HashMap::new(),
            main_js_embedded_dependency_name_counts: HashMap::new(),
            main_js_license_banner_count: None,
            main_js_credential_literal_count: None,
            main_js_api_capabilities: Vec::new(),
            main_js_api_disclosures: Vec::new(),
            latest_release_tag: None,
            latest_release_published_at: None,
            latest_release_fetch_status: None,
            analysis_errors: self.analysis_errors,
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct MainJsResult {
    pub(super) parse_succeeded: Option<bool>,
    pub(super) tolerant_parse_required: Option<bool>,
    pub(super) estimated_target_es_version: Option<String>,
    pub(super) is_probably_minified: Option<bool>,
    pub(super) minification_score: Option<f32>,
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
