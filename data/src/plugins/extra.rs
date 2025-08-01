use std::{fs, path::Path};

use data_lib::{
    input_data::{ObsCommunityPluginDeprecations, ObsCommunityPluginRemoved},
    plugin::{
        LicenseInfo, PluginData, PluginExtraData, PluginRepoData, bundlers::Bundler,
        packages::PackageManager, testing::TestingFramework,
    },
};
use hashbrown::HashMap;
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    constants::{
        OBS_RELEASES_REPO_PATH, PLUGIN_DEPRECATIONS_PATH, PLUGIN_REMOVED_PATH,
        PLUGIN_REPO_DATA_PATH, PLUGIN_REPO_PATH,
    },
    file_utils::{empty_dir, write_in_chunks},
    plugins::{data::read_plugin_data, license::license_compare::LicenseComparer},
};

pub fn extract_extra_data() -> Result<(), Box<dyn std::error::Error>> {
    let plugin_removed_list = get_removed_plugins()?;
    let plugin_version_deprecations = get_plugin_version_deprecations()?;

    let plugin_data = read_plugin_data()?;
    let mut license_comparer = LicenseComparer::new();
    license_comparer.init();

    let extra_data = plugin_data
        .par_iter()
        .progress_count(plugin_data.len() as u64)
        .map(|plugin| {
            let removed_entry = plugin_removed_list.iter().find(|r| r.id == plugin.id);
            let removal_reason = removed_entry.map(|r| r.reason.clone());

            let deprecated_versions = plugin_version_deprecations
                .0
                .get(&plugin.id)
                .map_or_else(Vec::new, |d| d.clone());

            let repo = if plugin.removed_commit.is_none() {
                extract_data_from_repo(plugin, &license_comparer)
            } else {
                Err(format!(
                    "Plugin {} was removed, skipping repository extraction",
                    plugin.id
                ))
            };

            PluginExtraData {
                id: plugin.id.clone(),
                repo,
                removal_reason,
                deprecated_versions,
            }
        })
        .collect::<Vec<PluginExtraData>>();

    empty_dir(Path::new(PLUGIN_REPO_DATA_PATH))?;

    write_in_chunks(Path::new(PLUGIN_REPO_DATA_PATH), &extra_data, 50)?;

    Ok(())
}

pub fn extract_data_from_repo(
    plugin: &PluginData,
    license_comparer: &LicenseComparer,
) -> Result<PluginRepoData, String> {
    let repo_path = format!("{}/{}", PLUGIN_REPO_PATH, plugin.id);
    if !std::path::Path::new(&repo_path).exists() {
        return Err(format!(
            "Repository for plugin {} does not exist at {}",
            plugin.id, repo_path
        ));
    }

    let manifest = fs::read_to_string(format!("{repo_path}/manifest.json"))
        .map_err(|e| format!("Failed to read manifest for plugin {}: {}", plugin.id, e))?;
    let manifest = match serde_json::from_str(&manifest) {
        Ok(manifest) => manifest,
        Err(e) => {
            return Err(format!(
                "Failed to parse manifest for plugin {}: {}",
                plugin.id, e
            ));
        }
    };

    let files = list_files_in_repo(&repo_path);

    let package_managers = PackageManager::find_package_managers(&files);
    let has_test_files = has_test_files(&files);
    let file_type_counts = count_file_types(&files);
    let uses_typescript =
        file_type_counts.contains_key("ts") || file_type_counts.contains_key("tsx");
    let has_beta_manifest = files.contains(&"manifest-beta.json".to_string());
    let has_package_json = files.contains(&"package.json".to_string());

    let mut dependencies = Vec::new();
    let mut dev_dependencies = Vec::new();
    let mut testing_frameworks = Vec::new();
    let mut bundlers = Vec::new();
    let mut package_json_license = LicenseInfo::NotFound;

    if has_package_json {
        let package_json =
            fs::read_to_string(format!("{repo_path}/package.json")).map_err(|e| {
                format!(
                    "Failed to read package.json for plugin {}: {}",
                    plugin.id, e
                )
            })?;
        let package_json: serde_json::Value = serde_json::from_str(&package_json).map_err(|e| {
            format!(
                "Failed to parse package.json for plugin {}: {}",
                plugin.id, e
            )
        })?;

        dependencies = package_json
            .get("dependencies")
            .and_then(|d| d.as_object())
            .map_or_else(Vec::new, |deps| deps.keys().cloned().collect());

        dev_dependencies = package_json
            .get("devDependencies")
            .and_then(|d| d.as_object())
            .map_or_else(Vec::new, |dev_deps| dev_deps.keys().cloned().collect());

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
    }

    let license_file = files.iter().find(|file| {
        let lower_case_file = file.to_lowercase();
        lower_case_file == "license"
            || lower_case_file == "license.txt"
            || lower_case_file == "license.md"
    });

    let file_license = license_file
        .and_then(|file| {
            fs::read_to_string(format!("{repo_path}/{file}"))
                .map(|license_text| license_comparer.compare(&plugin.id, &license_text))
                .ok()
        })
        .into();

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
    })
}

fn count_file_types(files: &[String]) -> HashMap<String, usize> {
    let mut file_types = HashMap::new();
    for file in files {
        if let Some(ext) = file.split('.').next_back() {
            *file_types.entry(ext.to_string()).or_insert(0) += 1;
        }
    }
    file_types
}

fn has_test_files(files: &[String]) -> bool {
    files.iter().any(|file| {
        file.ends_with(".test.ts")
            || file.ends_with(".test.js")
            || file.ends_with(".spec.ts")
            || file.ends_with(".spec.js")
    })
}

fn list_files_in_repo(repo_path: &str) -> Vec<String> {
    let mut files = Vec::new();
    list_files_rec(repo_path, &mut files);
    files
}

fn list_files_rec(path: &str, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|f| f == ".git") {
                    continue; // Skip .git directory
                }
                if path.file_name().is_some_and(|f| f == "node_modules") {
                    continue; // Skip node_modules directory
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

fn get_removed_plugins() -> Result<Vec<ObsCommunityPluginRemoved>, Box<dyn std::error::Error>> {
    let plugin_removed_list = fs::read_to_string(Path::new(&format!(
        "{OBS_RELEASES_REPO_PATH}/{PLUGIN_REMOVED_PATH}",
    )))
    .expect("Failed to read plugin removed list");

    Ok(serde_json::from_str(&plugin_removed_list)?)
}

fn get_plugin_version_deprecations()
-> Result<ObsCommunityPluginDeprecations, Box<dyn std::error::Error>> {
    let plugin_deprecations = fs::read_to_string(Path::new(&format!(
        "{OBS_RELEASES_REPO_PATH}/{PLUGIN_DEPRECATIONS_PATH}",
    )))
    .expect("Failed to read plugin deprecations");

    Ok(serde_json::from_str(&plugin_deprecations)?)
}
