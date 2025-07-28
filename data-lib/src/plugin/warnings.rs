use serde::Serialize;
use tsify::Tsify;

use crate::{commit::StringCommit, date::Date, plugin::full::FullPluginData};

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub enum PluginWarningSeverity {
    CAUTION,
    DANGER,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
#[serde(tag = "id")]
pub enum PluginWarning {
    Inactivity12Months(PluginWarningInactivity),
    Inactivity24Months(PluginWarningInactivity),
    Removed(PluginWarningRemoved),
    MismatchedManifestData(PluginWarningMismatchedManifestData),
    Unlicensed(PluginWarningUnlicensed),
    NoLicense(PluginWarningNoLicense),
    MismatchedLicense(PluginWarningMismatchedLicense),
    MissingExtendedData(PLuginWarningMissingExtendedData),
    MissingRepoData(PluginWarningMissingRepoData),
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginWarningInactivity {
    pub severity: PluginWarningSeverity,
    pub last_release_date: String,
    pub latest_version: String,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginWarningRemoved {
    pub severity: PluginWarningSeverity,
    pub removed_commit: StringCommit,
    pub removed_reason: Option<String>,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginWarningMismatchedManifestData {
    pub severity: PluginWarningSeverity,
    pub data: Vec<PluginWarningMismatchedManifestDataField>,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginWarningMismatchedManifestDataField {
    pub field: String,
    pub manifest_value: String,
    pub community_list_value: String,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginWarningUnlicensed {
    pub severity: PluginWarningSeverity,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginWarningNoLicense {
    pub severity: PluginWarningSeverity,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginWarningMismatchedLicense {
    pub severity: PluginWarningSeverity,
    pub license_file: String,
    pub package_json_license: String,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PLuginWarningMissingExtendedData {
    pub severity: PluginWarningSeverity,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginWarningMissingRepoData {
    pub severity: PluginWarningSeverity,

    pub message: String,
}

pub fn get_plugin_warnings(data: &FullPluginData) -> Vec<PluginWarning> {
    let mut warnings = Vec::new();

    get_missing_warnings(data, &mut warnings);
    get_manifest_warnings(data, &mut warnings);
    get_license_warnings(data, &mut warnings);
    get_inactivity_warnings(data, &mut warnings);

    warnings
}

fn get_inactivity_warnings(data: &FullPluginData, warnings: &mut Vec<PluginWarning>) {
    if let Some(commit) = &data.data.removed_commit {
        warnings.push(PluginWarning::Removed(PluginWarningRemoved {
            severity: PluginWarningSeverity::DANGER,
            removed_commit: commit.to_string_commit(),
            removed_reason: data
                .extended
                .as_ref()
                .and_then(|e| e.removal_reason.clone()),
        }));
        return;
    }

    let latest_release_date = data
        .data
        .version_history
        .last()
        .map(|v| &v.initial_release_date)
        .unwrap_or(&data.data.added_commit.date);

    let latest_version = data.data.version_history.last().map(|v| &v.version);

    let now = Date::now();
    let one_year_ago = {
        let mut date = now.clone();
        date.reverse_days(365);
        date
    };
    let two_years_ago = {
        let mut date = now.clone();
        date.reverse_days(365 * 2);
        date
    };

    if latest_release_date < &two_years_ago {
        warnings.push(PluginWarning::Inactivity24Months(PluginWarningInactivity {
            severity: PluginWarningSeverity::DANGER,
            last_release_date: latest_release_date.to_fancy_string(),
            latest_version: latest_version.map_or_else(|| "Unknown".to_string(), |v| v.clone()),
        }));
    } else if latest_release_date < &one_year_ago {
        warnings.push(PluginWarning::Inactivity12Months(PluginWarningInactivity {
            severity: PluginWarningSeverity::CAUTION,
            last_release_date: latest_release_date.to_fancy_string(),
            latest_version: latest_version.map_or_else(|| "Unknown".to_string(), |v| v.clone()),
        }));
    }
}

fn get_manifest_warnings(data: &FullPluginData, warnings: &mut Vec<PluginWarning>) {
    let manifest = if let Some(extended) = &data.extended
        && let Ok(repo) = &extended.repo
    {
        &repo.manifest
    } else {
        return;
    };

    let data_to_check = [
        ("name", &manifest.name, &data.data.current_entry.name),
        (
            "description",
            &manifest.description,
            &data.data.current_entry.description,
        ),
        ("author", &manifest.author, &data.data.current_entry.author),
    ];

    let mismatched_data = data_to_check
        .into_iter()
        .filter(|(_, manifest_value, community_value)| manifest_value != community_value)
        .collect::<Vec<_>>();

    if !mismatched_data.is_empty() {
        warnings.push(PluginWarning::MismatchedManifestData(
            PluginWarningMismatchedManifestData {
                severity: PluginWarningSeverity::CAUTION,
                data: mismatched_data
                    .into_iter()
                    .map(|(field, manifest_value, community_value)| {
                        PluginWarningMismatchedManifestDataField {
                            field: field.to_string(),
                            manifest_value: manifest_value.clone(),
                            community_list_value: community_value.clone(),
                        }
                    })
                    .collect(),
            },
        ));
    }
}

fn get_license_warnings(data: &FullPluginData, warnings: &mut Vec<PluginWarning>) {
    let repo = if let Some(extended) = &data.extended
        && let Ok(repo) = &extended.repo
    {
        repo
    } else {
        return;
    };

    if repo.license_file == "explicitly unlicensed" {
        warnings.push(PluginWarning::Unlicensed(PluginWarningUnlicensed {
            severity: PluginWarningSeverity::CAUTION,
        }));
    } else if repo.license_file == "no license" {
        warnings.push(PluginWarning::NoLicense(PluginWarningNoLicense {
            severity: PluginWarningSeverity::CAUTION,
        }));
    } else if repo.license_file != "unknown"
        && repo.license_file != "not found"
        && repo.package_json_license != "unknown"
        && repo.package_json_license != "not found"
        && repo.package_json_license != "no license"
        && !repo.package_json_license.starts_with(&repo.license_file)
        && !repo.license_file.starts_with(&repo.package_json_license)
    {
        warnings.push(PluginWarning::MismatchedLicense(
            PluginWarningMismatchedLicense {
                severity: PluginWarningSeverity::CAUTION,
                license_file: repo.license_file.clone(),
                package_json_license: repo.package_json_license.clone(),
            },
        ));
    }
}

fn get_missing_warnings(data: &FullPluginData, warnings: &mut Vec<PluginWarning>) {
    if data.extended.is_none() {
        warnings.push(PluginWarning::MissingExtendedData(
            PLuginWarningMissingExtendedData {
                severity: PluginWarningSeverity::DANGER,
            },
        ));
    }

    if let Some(extended) = &data.extended
        && let Err(e) = &extended.repo
    {
        warnings.push(PluginWarning::MissingRepoData(
            PluginWarningMissingRepoData {
                severity: PluginWarningSeverity::DANGER,
                message: e.clone(),
            },
        ));
    }
}
