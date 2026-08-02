use data_lib::plugin::{
    LicenseInfo, PluginManifest, PluginRepoAnalysisError, PluginRepoData, bundlers::Bundler,
    packages::PackageManager, testing::TestingFramework,
};
use hashbrown::HashMap;

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
            estimated_target_es_version: None,
            main_js_is_probably_minified: None,
            main_js_minification_score: None,
            main_js_findings: Vec::new(),
            main_js_disclosures: Vec::new(),
            main_js_diagnostics: Vec::new(),
            latest_release_tag: None,
            latest_release_published_at: None,
            latest_release_fetch_status: None,
            analysis_errors: self.analysis_errors,
        }
    }
}
