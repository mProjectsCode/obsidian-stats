use std::{fs, io, path::Path};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{constants::{OBS_RELEASES_REPO_PATH, PLUGIN_DEPRECATIONS_PATH, PLUGIN_REMOVED_PATH}, input_data::{ObsCommunityPluginDeprecations, ObsCommunityPluginRemoved}, plugins::{
    data::read_plugin_data, license::license_compare::LicenseComparer, repo::{
        bundlers::Bundler, packages::PackageManager, testing::TestingFramework, PluginManifest, PluginRepoData, PluginRepoExtractedData
    }, SerializedPluginData
}};

pub fn extract_repo_data() -> io::Result<()> {
    fs::remove_dir_all(Path::new("../pluginRepos/data"))?;
    fs::create_dir(Path::new("../pluginRepos/data"))?;

    let plugin_removed_list = fs::read_to_string(Path::new(&format!("{}/{}", OBS_RELEASES_REPO_PATH, PLUGIN_REMOVED_PATH)))?;
    let plugin_removed_list: Vec<ObsCommunityPluginRemoved> = serde_json::from_str(&plugin_removed_list)
        .expect("Failed to parse plugin removed list");

    let plugin_deprecations = fs::read_to_string(Path::new(&format!("{}/{}", OBS_RELEASES_REPO_PATH, PLUGIN_DEPRECATIONS_PATH)))?;
    let plugin_deprecations: ObsCommunityPluginDeprecations = serde_json::from_str(&plugin_deprecations)
        .expect("Failed to parse plugin deprecations");

    let plugin_data = read_plugin_data();
    let mut license_comparer = LicenseComparer::new();
    license_comparer.init();

    plugin_data.par_iter().for_each(|plugin| {
        let mut repo_data = PluginRepoData {
            id: plugin.id.clone(),
            repo: None,
            warnings: Vec::new(),
            removal_reason: None,
            deprecated_versions: Vec::new(),
        };

        let removed_entry = plugin_removed_list.iter().find(|r| r.id == plugin.id);
        if let Some(removed) = removed_entry {
            repo_data.removal_reason = Some(removed.reason.clone());
        }

        if let Some(deprecation) = plugin_deprecations.0.get(&plugin.id) {
            repo_data.deprecated_versions = deprecation.clone();
        }

        if plugin.removed_commit.is_none() {
            repo_data.repo = extract_data_from_repo(&plugin, &license_comparer);
        }

        // TODO: warnings

        let data_string = serde_json::to_string(&repo_data)
            .expect("Failed to serialize plugin repo data");

        fs::write(Path::new(&format!("../pluginRepos/data/{}.json", plugin.id)), data_string)
            .expect("Failed to write plugin repo data to file");
    });

    Ok(())
}

pub fn extract_data_from_repo(
    plugin: &SerializedPluginData,
    license_comparer: &LicenseComparer,
) -> Option<PluginRepoExtractedData> {
    let repo_path = format!("../pluginRepos/repos/{}", plugin.id);
    if !std::path::Path::new(&repo_path).exists() {
        println!("Repository for plugin {} does not exist at {}", plugin.id, repo_path);
        return None;
    }

    let manifest = fs::read_to_string(format!("{}/manifest.json", repo_path)).ok()?;
    let manifest = match serde_json::from_str(&manifest) {
        Ok(manifest) => manifest,
        Err(e) => {
            println!("Failed to parse manifest for plugin {}: {}", plugin.id, e);
            return None;
        }
    };

    let files = list_files_in_repo(&repo_path);

    let package_managers = PackageManager::find_package_managers(&files);
    let has_test_files = has_test_files(&files);
    let file_type_counts = count_file_types(&files);
    let uses_typescript =
        file_type_counts.get("ts").is_some() || file_type_counts.get("tsx").is_some();
    let has_beta_manifest = files.contains(&"manifest-beta.json".to_string());
    let has_package_json = files.contains(&"package.json".to_string());

    let mut dependencies = Vec::new();
    let mut dev_dependencies = Vec::new();
    let mut testing_frameworks = Vec::new();
    let mut bundlers = Vec::new();
    let mut package_json_license = "unknown".to_string();

    if has_package_json {
        let package_json = fs::read_to_string(format!("{}/package.json", repo_path)).ok()?;
        let package_json: serde_json::Value = serde_json::from_str(&package_json).ok()?;

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
            .and_then(|l| if l == "" { None } else { Some(l.to_string()) })
            .unwrap_or("not set".to_string());
    }

    let license_file = files.iter().find(|file| {
        let lower_case_file = file.to_lowercase();
        lower_case_file == "license"
            || lower_case_file == "license.txt"
            || lower_case_file == "license.md"
    });

    let license_file = license_file.and_then(|file| {
        let license_text = fs::read_to_string(format!("{}/{}", repo_path, file)).ok()?;
        license_comparer.compare(&license_text)
    });

    let license_file = license_file.unwrap_or_else(|| "unknown".to_string());

    Some(PluginRepoExtractedData {
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
        license_file,
        manifest,
    })
}

fn count_file_types(files: &[String]) -> std::collections::HashMap<String, usize> {
    let mut file_types = std::collections::HashMap::new();
    for file in files {
        if let Some(ext) = file.split('.').last() {
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
                if path.file_name().map_or(false, |f| f == ".git") {
                    continue; // Skip .git directory
                }
                if path.file_name().map_or(false, |f| f == "node_modules") {
                    continue; // Skip node_modules directory
                }

                list_files_rec(path.to_str().unwrap(), files);
            } else if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    files.push(file_name.to_string_lossy().to_string());
                }
            }
        }
    }
}
