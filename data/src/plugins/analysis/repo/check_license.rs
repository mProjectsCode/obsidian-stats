use std::fs;

use data_lib::plugin::LicenseInfo;

use crate::plugins::license::license_compare::LicenseComparer;

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

    let license_file = files.iter().find(|file| {
        let lower_case_file = file.to_lowercase();
        LICENSE_FILE_CANDIDATES
            .iter()
            .any(|candidate| lower_case_file == *candidate)
    });

    let file_license = license_file
        .and_then(|file| {
            fs::read_to_string(format!("{repo_path}/{file}"))
                .map(|license_text| license_comparer.compare(plugin_id, &license_text))
                .ok()
        })
        .into();

    (package_json_license, file_license)
}
