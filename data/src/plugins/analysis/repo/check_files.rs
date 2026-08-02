use std::{
    fs,
    path::{Path, PathBuf},
};

use data_lib::plugin::{PluginRepoAnalysisError, packages::PackageManager};
use hashbrown::HashMap;

use super::{LOC_EXCLUDED, check_i18n, check_license};

const TEST_FILE_SUFFIXES: &[&str] = &[".test.ts", ".test.js", ".spec.ts", ".spec.js"];
/// The bounded summary produced while walking a repository. It deliberately
/// contains only the final language-count map after Tokei has scanned the
/// repository once.
pub(super) struct RepoFileSummary {
    pub(super) package_managers: Vec<PackageManager>,
    pub(super) file_type_counts: HashMap<String, usize>,
    pub(super) has_test_files: bool,
    pub(super) has_beta_manifest: bool,
    pub(super) has_package_json: bool,
    pub(super) uses_typescript: bool,
    pub(super) lines_of_code: HashMap<String, usize>,
    pub(super) i18n_files: bool,
    pub(super) license_file: Option<String>,
    pub(super) analysis_errors: Vec<PluginRepoAnalysisError>,
}

impl Default for RepoFileSummary {
    fn default() -> Self {
        Self {
            package_managers: Vec::new(),
            file_type_counts: HashMap::new(),
            has_test_files: false,
            has_beta_manifest: false,
            has_package_json: false,
            uses_typescript: false,
            lines_of_code: HashMap::new(),
            i18n_files: false,
            license_file: None,
            analysis_errors: Vec::new(),
        }
    }
}

pub(super) fn run(repo_path: &str) -> RepoFileSummary {
    let mut summary = RepoFileSummary::default();
    let mut tokei_paths = Vec::new();
    let mut had_scan_errors = false;
    let root = Path::new(repo_path);
    visit_files(
        root,
        root,
        &mut summary,
        &mut tokei_paths,
        &mut had_scan_errors,
    );
    count_lines_of_code(&mut summary.lines_of_code, &mut tokei_paths);

    summary.uses_typescript =
        summary.file_type_counts.contains_key("ts") || summary.file_type_counts.contains_key("tsx");
    if had_scan_errors {
        summary
            .analysis_errors
            .push(PluginRepoAnalysisError::RepositoryScan);
    }
    summary
}

fn visit_files(
    root: &Path,
    path: &Path,
    summary: &mut RepoFileSummary,
    tokei_paths: &mut Vec<PathBuf>,
    had_scan_errors: &mut bool,
) {
    let Ok(entries) = fs::read_dir(path) else {
        *had_scan_errors = true;
        return;
    };

    for entry in entries {
        let Ok(entry) = entry else {
            *had_scan_errors = true;
            continue;
        };
        let path = entry.path();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            *had_scan_errors = true;
            continue;
        };
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name == ".git" || name == "node_modules")
            {
                continue;
            }
            visit_files(root, &path, summary, tokei_paths, had_scan_errors);
        } else if file_type.is_file()
            && let Some(relative_path) = repo_relative_path(root, &path)
        {
            record_file(summary, &relative_path);
            tokei_paths.push(path);
        }
    }
}

fn record_file(summary: &mut RepoFileSummary, file: &str) {
    if let Some(ext) = Path::new(file).extension().and_then(|ext| ext.to_str()) {
        *summary
            .file_type_counts
            .entry(ext.to_lowercase())
            .or_insert(0) += 1;
    }

    summary.has_test_files |= TEST_FILE_SUFFIXES
        .iter()
        .any(|suffix| file.ends_with(suffix));
    summary.has_beta_manifest |= file == "manifest-beta.json";
    summary.has_package_json |= file == "package.json";
    summary.i18n_files |= check_i18n::is_i18n_file(file);

    for package_manager in PackageManager::iter_variants() {
        if package_manager.matches_lock_file(file)
            && !summary.package_managers.contains(&package_manager)
        {
            summary.package_managers.push(package_manager);
        }
    }

    if check_license::is_license_candidate(file) {
        let is_better = summary.license_file.as_ref().is_none_or(|current| {
            let depth = file.matches('/').count();
            let current_depth = current.matches('/').count();
            depth < current_depth || (depth == current_depth && file < current.as_str())
        });
        if is_better {
            summary.license_file = Some(file.to_string());
        }
    }
}

fn count_lines_of_code(totals: &mut HashMap<String, usize>, paths: &[PathBuf]) {
    if paths.is_empty() {
        return;
    }

    let config = tokei::Config::default();
    let mut languages = tokei::Languages::new();
    languages.get_statistics(paths, LOC_EXCLUDED, &config);
    for (language, stats) in languages {
        if stats.code > 0 {
            *totals.entry(language.name().into()).or_insert(0) += stats.code;
        }
    }
}

fn repo_relative_path(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root).ok().map(|relative| {
        relative
            .components()
            .filter_map(|component| component.as_os_str().to_str())
            .collect::<Vec<_>>()
            .join("/")
    })
}
