use std::fs;

use data_lib::plugin::LicenseInfo;

use crate::plugins::license::license_compare::LicenseComparer;

use super::safe_repo_file_path;

const LICENSE_FILE_CANDIDATES: &[&str] = &["license", "license.txt", "license.md"];

pub(super) fn run(
    plugin_id: &str,
    repo_path: &str,
    files: &[String],
    package_json_license: Option<&str>,
    license_comparer: &LicenseComparer,
) -> (LicenseInfo, LicenseInfo) {
    let package_json_license = package_json_license
        .map(|license| {
            if license.is_empty() {
                LicenseInfo::Unrecognized
            } else {
                LicenseInfo::Known(license.to_string())
            }
        })
        .into();

    let license_file = files
        .iter()
        .filter(|file| {
            let lower_case_file = file.rsplit('/').next().unwrap_or(file).to_lowercase();
            LICENSE_FILE_CANDIDATES
                .iter()
                .any(|candidate| lower_case_file == *candidate)
        })
        .min_by_key(|file| file.matches('/').count());

    let file_license = license_file
        .and_then(|file| {
            let path = safe_repo_file_path(repo_path, file).ok()?;
            fs::read_to_string(path)
                .map(|license_text| license_comparer.compare(plugin_id, &license_text))
                .ok()
        })
        .into();

    (package_json_license, file_license)
}
