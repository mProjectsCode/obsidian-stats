use std::{fs, path::Path};

use hashbrown::HashMap;

use super::LOC_EXCLUDED;

const TEST_FILE_SUFFIXES: &[&str] = &[".test.ts", ".test.js", ".spec.ts", ".spec.js"];

pub(super) struct FilesResult {
    pub(super) files: Vec<String>,
    pub(super) file_type_counts: HashMap<String, usize>,
    pub(super) has_test_files: bool,
    pub(super) has_beta_manifest: bool,
    pub(super) has_package_json: bool,
    pub(super) uses_typescript: bool,
    pub(super) lines_of_code: HashMap<String, usize>,
}

pub(super) fn run(repo_path: &str) -> FilesResult {
    let files = list_files_in_repo(repo_path);
    let file_type_counts = count_file_types(&files);

    let uses_typescript =
        file_type_counts.contains_key("ts") || file_type_counts.contains_key("tsx");

    FilesResult {
        has_test_files: has_test_files(&files),
        has_beta_manifest: has_file_named(&files, "manifest-beta.json"),
        has_package_json: has_file_named(&files, "package.json"),
        lines_of_code: count_lines_of_code(repo_path),
        files,
        file_type_counts,
        uses_typescript,
    }
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

                if let Some(path_str) = path.to_str() {
                    list_files_rec(path_str, files);
                }
            } else if path.is_file()
                && let Some(file_name) = path.file_name()
            {
                files.push(file_name.to_string_lossy().to_string());
            }
        }
    }
}
