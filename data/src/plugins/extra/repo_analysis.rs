use std::{fs, path::Path};

use data_lib::{
    common::{I18N_DEPENDENCIES, I18N_FILE_ENDINGS, I18N_LOCALE_CODES},
    input_data::{ObsCommunityPluginDeprecations, ObsCommunityPluginRemoved},
    plugin::{
        LicenseInfo, PluginData, PluginManifest, PluginRepoData, bundlers::Bundler,
        packages::PackageManager, testing::TestingFramework,
    },
};
use hashbrown::HashMap;

use super::LOC_EXCLUDED;
use crate::{
    constants::{
        OBS_RELEASES_REPO_PATH, PLUGIN_DEPRECATIONS_PATH, PLUGIN_REMOVED_PATH, PLUGIN_REPO_PATH,
    },
    plugins::license::license_compare::LicenseComparer,
};

const LICENSE_FILE_CANDIDATES: &[&str] = &["license", "license.txt", "license.md"];
const TEST_FILE_SUFFIXES: &[&str] = &[".test.ts", ".test.js", ".spec.ts", ".spec.js"];

pub(super) fn analyze_plugin_repository(
    plugin: &PluginData,
    license_comparer: &LicenseComparer,
) -> Result<PluginRepoData, String> {
    let repo_path = format!("{}/{}", PLUGIN_REPO_PATH, plugin.id);
    if !Path::new(&repo_path).exists() {
        return Err(format!(
            "Repository for plugin {} does not exist at {}",
            plugin.id, repo_path
        ));
    }

    let manifest = read_manifest(&repo_path, &plugin.id)?;

    let files = list_files_in_repo(&repo_path);

    let package_managers = PackageManager::find_package_managers(&files);
    let has_test_files = has_test_files(&files);
    let file_type_counts = count_file_types(&files);
    let uses_typescript =
        file_type_counts.contains_key("ts") || file_type_counts.contains_key("tsx");
    let has_beta_manifest = has_file_named(&files, "manifest-beta.json");
    let has_package_json = has_file_named(&files, "package.json");

    let mut dependencies = Vec::new();
    let mut dev_dependencies = Vec::new();
    let mut testing_frameworks = Vec::new();
    let mut bundlers = Vec::new();
    let mut package_json_license = LicenseInfo::NotFound;

    let mut has_i18n_dependencies = false;

    if has_package_json {
        let package_json = read_package_json(&repo_path, &plugin.id)?;

        dependencies = sorted_object_keys(&package_json, "dependencies");
        dev_dependencies = sorted_object_keys(&package_json, "devDependencies");

        let all_dependencies = dependencies
            .iter()
            .chain(dev_dependencies.iter())
            .collect::<Vec<_>>();

        testing_frameworks = TestingFramework::find_testing_frameworks(&all_dependencies);
        bundlers = Bundler::find_bundlers(&all_dependencies);
        package_json_license = package_json
            .get("license")
            .and_then(|l| l.as_str())
            .map(|l| {
                if l.is_empty() {
                    LicenseInfo::Unrecognized
                } else {
                    LicenseInfo::Known(l.to_string())
                }
            })
            .into();

        has_i18n_dependencies = all_dependencies
            .iter()
            .any(|d| I18N_DEPENDENCIES.contains(&d.as_str()));
    }

    let license_file = files.iter().find(|file| {
        let lower_case_file = file.to_lowercase();
        LICENSE_FILE_CANDIDATES
            .iter()
            .any(|candidate| lower_case_file == *candidate)
    });

    let file_license = license_file
        .and_then(|file| {
            fs::read_to_string(format!("{repo_path}/{file}"))
                .map(|license_text| license_comparer.compare(&plugin.id, &license_text))
                .ok()
        })
        .into();

    let lines_of_code = count_lines_of_code(&repo_path);

    let has_i18n_files = files.iter().any(|file| {
        I18N_LOCALE_CODES.iter().any(|code| {
            I18N_FILE_ENDINGS
                .iter()
                .any(|ending| file == &format!("{code}{ending}"))
        })
    });

    Ok(PluginRepoData {
        uses_typescript,
        has_package_json,
        package_managers,
        dependencies,
        dev_dependencies,
        testing_frameworks,
        bundlers,
        has_test_files,
        has_beta_manifest,
        file_type_counts,
        package_json_license,
        file_license,
        manifest,
        lines_of_code,
        has_i18n_dependencies,
        has_i18n_files,
        latest_release_main_js_size_bytes: None,
        estimated_target_es_version: None,
        latest_release_tag: None,
        latest_release_published_at: None,
        latest_release_fetch_status: None,
    })
}

fn count_file_types(files: &[String]) -> HashMap<String, usize> {
    let mut file_types = HashMap::new();
    for file in files {
        if let Some(ext) = Path::new(file).extension().and_then(|ext| ext.to_str()) {
            *file_types.entry(ext.to_lowercase()).or_insert(0) += 1;
        }
    }
    file_types
}

fn count_lines_of_code(repo_path: &str) -> HashMap<String, usize> {
    let config = tokei::Config::default();
    let mut languages = tokei::Languages::new();

    languages.get_statistics(&[repo_path], LOC_EXCLUDED, &config);

    languages
        .into_iter()
        .map(|(lang, stats)| (lang.name().into(), stats.code))
        .filter(|(_, count)| *count > 0)
        .collect()
}

fn has_test_files(files: &[String]) -> bool {
    files.iter().any(|file| {
        TEST_FILE_SUFFIXES
            .iter()
            .any(|suffix| file.ends_with(suffix))
    })
}

fn list_files_in_repo(repo_path: &str) -> Vec<String> {
    let mut files = Vec::new();
    list_files_rec(repo_path, &mut files);
    files
}

fn has_file_named(files: &[String], target: &str) -> bool {
    files.iter().any(|file| file == target)
}

fn read_manifest(repo_path: &str, plugin_id: &str) -> Result<PluginManifest, String> {
    let manifest = fs::read_to_string(format!("{repo_path}/manifest.json"))
        .map_err(|e| format!("Failed to read manifest for plugin {}: {}", plugin_id, e))?;

    serde_json::from_str(&manifest)
        .map_err(|e| format!("Failed to parse manifest for plugin {}: {}", plugin_id, e))
}

fn read_package_json(repo_path: &str, plugin_id: &str) -> Result<serde_json::Value, String> {
    let package_json = fs::read_to_string(format!("{repo_path}/package.json")).map_err(|e| {
        format!(
            "Failed to read package.json for plugin {}: {}",
            plugin_id, e
        )
    })?;

    serde_json::from_str(&package_json).map_err(|e| {
        format!(
            "Failed to parse package.json for plugin {}: {}",
            plugin_id, e
        )
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

fn list_files_rec(path: &str, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|f| f == ".git") {
                    continue;
                }
                if path.file_name().is_some_and(|f| f == "node_modules") {
                    continue;
                }

                list_files_rec(path.to_str().unwrap(), files);
            } else if path.is_file()
                && let Some(file_name) = path.file_name()
            {
                files.push(file_name.to_string_lossy().to_string());
            }
        }
    }
}

pub(super) fn read_removed_plugins()
-> Result<Vec<ObsCommunityPluginRemoved>, Box<dyn std::error::Error>> {
    let plugin_removed_list = fs::read_to_string(Path::new(&format!(
        "{OBS_RELEASES_REPO_PATH}/{PLUGIN_REMOVED_PATH}",
    )))
    .map_err(|e| format!("Failed to read plugin removed list: {e}"))?;

    Ok(serde_json::from_str(&plugin_removed_list)?)
}

pub(super) fn read_plugin_version_deprecations()
-> Result<ObsCommunityPluginDeprecations, Box<dyn std::error::Error>> {
    let plugin_deprecations = fs::read_to_string(Path::new(&format!(
        "{OBS_RELEASES_REPO_PATH}/{PLUGIN_DEPRECATIONS_PATH}",
    )))
    .map_err(|e| format!("Failed to read plugin deprecations: {e}"))?;

    Ok(serde_json::from_str(&plugin_deprecations)?)
}
