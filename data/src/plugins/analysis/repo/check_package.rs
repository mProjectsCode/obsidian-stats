use std::fs;

use data_lib::{
    common::I18N_DEPENDENCIES,
    plugin::{
        PluginRepoAnalysisDetailError, bundlers::Bundler, packages::PackageManager,
        testing::TestingFramework,
    },
};

pub(super) struct PackageResult {
    pub(super) package_managers: Vec<PackageManager>,
    pub(super) dependencies: Vec<String>,
    pub(super) dev_dependencies: Vec<String>,
    pub(super) testing_frameworks: Vec<TestingFramework>,
    pub(super) bundlers: Vec<Bundler>,
    pub(super) package_json_license: Option<String>,
    pub(super) has_i18n_dependencies: bool,
}

impl PackageResult {
    pub(super) fn empty(files: &[String]) -> Self {
        Self {
            package_managers: PackageManager::find_package_managers(files),
            dependencies: Vec::new(),
            dev_dependencies: Vec::new(),
            testing_frameworks: Vec::new(),
            bundlers: Vec::new(),
            package_json_license: None,
            has_i18n_dependencies: false,
        }
    }
}

pub(super) fn run(
    repo_path: &str,
    plugin_id: &str,
    files: &[String],
) -> Result<PackageResult, PluginRepoAnalysisDetailError> {
    if !files.iter().any(|file| file == "package.json") {
        return Ok(PackageResult::empty(files));
    }

    let package_json = read_package_json(repo_path, plugin_id)?;
    let dependencies = sorted_object_keys(&package_json, "dependencies");
    let dev_dependencies = sorted_object_keys(&package_json, "devDependencies");

    let all_dependencies = dependencies
        .iter()
        .chain(dev_dependencies.iter())
        .collect::<Vec<_>>();

    let package_json_license = package_json
        .get("license")
        .and_then(|l| l.as_str())
        .map(|s| s.to_string());

    Ok(PackageResult {
        package_managers: PackageManager::find_package_managers(files),
        testing_frameworks: TestingFramework::find_testing_frameworks(&all_dependencies),
        bundlers: Bundler::find_bundlers(&all_dependencies),
        has_i18n_dependencies: all_dependencies
            .iter()
            .any(|dep| I18N_DEPENDENCIES.contains(&dep.as_str())),
        dependencies,
        dev_dependencies,
        package_json_license,
    })
}

fn read_package_json(
    repo_path: &str,
    plugin_id: &str,
) -> Result<serde_json::Value, PluginRepoAnalysisDetailError> {
    let package_json =
        fs::read_to_string(format!("{repo_path}/package.json")).map_err(|source| {
            PluginRepoAnalysisDetailError::PackageJsonRead {
                plugin_id: plugin_id.to_string(),
                source,
            }
        })?;

    let package_json = package_json.trim_start_matches('\u{feff}');

    serde_json::from_str(package_json).map_err(|source| {
        PluginRepoAnalysisDetailError::PackageJsonParse {
            plugin_id: plugin_id.to_string(),
            source,
        }
    })
}

fn sorted_object_keys(package_json: &serde_json::Value, key: &str) -> Vec<String> {
    let mut keys = package_json
        .get(key)
        .and_then(|value| value.as_object())
        .map_or_else(Vec::new, |obj| obj.keys().cloned().collect());

    keys.sort();
    keys
}
