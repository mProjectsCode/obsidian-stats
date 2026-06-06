use std::path::{Path, PathBuf};

use data_lib::plugin::{PluginData, PluginRepoAnalysisDetailError, PluginRepoData};
use hashbrown::HashMap;

use super::types::RepoResult;
use crate::{
    constants::PLUGIN_REPO_PATH,
    plugins::license::license_compare::LicenseComparer,
    security::{validate_existing_path_under, validate_relative_repo_path, validated_plugin_path},
};

mod check_files;
mod check_i18n;
mod check_license;
mod check_manifest;
mod check_package;

pub(super) const LOC_EXCLUDED: &[&str] = &[
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "bun.lockb",
    "bun.lock",
    "lock.json",
    "node_modules",
];

pub(super) fn analyze_repo(
    plugin: &PluginData,
    license_comparer: &LicenseComparer,
) -> Result<RepoResult, PluginRepoAnalysisDetailError> {
    let repo_path =
        validated_plugin_path(Path::new(PLUGIN_REPO_PATH), &plugin.id).map_err(|error| {
            PluginRepoAnalysisDetailError::RepositoryMissing {
                plugin_id: plugin.id.clone(),
                path: PathBuf::from(error),
            }
        })?;
    if !repo_path.exists() {
        return Err(PluginRepoAnalysisDetailError::RepositoryMissing {
            plugin_id: plugin.id.clone(),
            path: repo_path.clone(),
        });
    }
    let repo_path = repo_path.to_string_lossy().to_string();

    let file_data = check_files::run(&repo_path);
    let mut analysis_errors = file_data.analysis_errors.clone();

    let manifest = match check_manifest::run(&repo_path, &plugin.id) {
        Ok(manifest) => Some(manifest),
        Err(error) => {
            analysis_errors.push(error.code());
            None
        }
    };
    let package_data = match check_package::run(&repo_path, &plugin.id, &file_data.files) {
        Ok(package_data) => package_data,
        Err(error) => {
            analysis_errors.push(error.code());
            check_package::PackageResult::empty(&file_data.files)
        }
    };

    let (package_json_license, file_license) = check_license::run(
        &plugin.id,
        &repo_path,
        &file_data.files,
        package_data.package_json_license.as_deref(),
        license_comparer,
    );

    Ok(RepoResult {
        uses_typescript: file_data.uses_typescript,
        has_package_json: file_data.has_package_json,
        package_managers: package_data.package_managers,
        dependencies: package_data.dependencies,
        dev_dependencies: package_data.dev_dependencies,
        testing_frameworks: package_data.testing_frameworks,
        bundlers: package_data.bundlers,
        has_test_files: file_data.has_test_files,
        has_beta_manifest: file_data.has_beta_manifest,
        file_type_counts: file_data.file_type_counts,
        package_json_license,
        file_license,
        manifest,
        lines_of_code: file_data.lines_of_code,
        has_i18n_dependencies: package_data.has_i18n_dependencies,
        has_i18n_files: check_i18n::has_i18n_files(&file_data.files),
        analysis_errors,
    })
}

pub(super) fn safe_repo_file_path(
    repo_path: &str,
    relative_path: &str,
) -> Result<PathBuf, std::io::Error> {
    validate_relative_repo_path(relative_path).map_err(std::io::Error::other)?;

    let root = Path::new(repo_path);
    let path = root.join(relative_path);
    let metadata = std::fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink() {
        return Err(std::io::Error::other(format!(
            "refusing to read symlinked repository file: {relative_path}"
        )));
    }

    validate_existing_path_under(root, &path).map_err(std::io::Error::other)?;

    Ok(path)
}

pub(super) fn into_plugin_repo_data(result: RepoResult) -> PluginRepoData {
    PluginRepoData {
        uses_typescript: result.uses_typescript,
        has_package_json: result.has_package_json,
        package_managers: result.package_managers,
        testing_frameworks: result.testing_frameworks,
        bundlers: result.bundlers,
        dependencies: result.dependencies,
        dev_dependencies: result.dev_dependencies,
        has_test_files: result.has_test_files,
        has_beta_manifest: result.has_beta_manifest,
        file_type_counts: result.file_type_counts,
        package_json_license: result.package_json_license,
        file_license: result.file_license,
        manifest: result.manifest,
        lines_of_code: result.lines_of_code,
        has_i18n_dependencies: result.has_i18n_dependencies,
        has_i18n_files: result.has_i18n_files,
        latest_release_main_js_size_bytes: None,
        main_js_parse_succeeded: None,
        main_js_tolerant_parse_required: None,
        estimated_target_es_version: None,
        main_js_is_probably_minified: None,
        main_js_minification_score: None,
        main_js_includes_sourcemap_comment: None,
        main_js_includes_inline_sourcemap: None,
        main_js_large_base64_blob_count: None,
        main_js_largest_base64_blob_length: None,
        main_js_embedded_blob_type_counts: HashMap::new(),
        main_js_worker_usage_count: None,
        main_js_webassembly_usage_count: None,
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
        analysis_errors: result.analysis_errors,
    }
}
