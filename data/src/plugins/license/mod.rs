use std::collections::HashSet;
use std::error::Error;

use data_lib::license::{LicenseData, LicenseDescriptionNested, Licenses};
use serde_yaml;

pub mod license_compare;

pub fn process_licenses() -> Result<(), Box<dyn Error>> {
    let dir = std::fs::read_dir("../choosealicense.com/_licenses")?;
    let mut skipped_files = 0usize;
    let licenses: Vec<LicenseData> = dir
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_file() {
                    let data = match std::fs::read_to_string(&path) {
                        Ok(data) => data,
                        Err(_) => {
                            skipped_files += 1;
                            return None;
                        }
                    };
                    let parts: Vec<&str> = data.split("---").collect();
                    if parts.len() < 2 {
                        skipped_files += 1;
                        return None;
                    }
                    match serde_yaml::from_str(parts[1]) {
                        Ok(license) => Some(license),
                        Err(_) => {
                            skipped_files += 1;
                            None
                        }
                    }
                } else {
                    None
                }
            })
        })
        .collect();

    let mut all_permissions: HashSet<String> = HashSet::new();
    let mut all_conditions: HashSet<String> = HashSet::new();
    let mut all_limitations: HashSet<String> = HashSet::new();

    for license in &licenses {
        for permission in &license.permissions {
            all_permissions.insert(permission.clone());
        }
        for condition in &license.conditions {
            all_conditions.insert(condition.clone());
        }
        for limitation in &license.limitations {
            all_limitations.insert(limitation.clone());
        }
    }

    let rules_data = std::fs::read_to_string("../choosealicense.com/_data/rules.yml")?;
    let descriptions: LicenseDescriptionNested = serde_yaml::from_str(&rules_data)?;

    let licenses_data = Licenses {
        licenses,
        permissions: all_permissions.into_iter().collect(),
        conditions: all_conditions.into_iter().collect(),
        limitations: all_limitations.into_iter().collect(),
        descriptions,
    };

    let licenses_json = serde_json::to_string_pretty(&licenses_data)?;
    std::fs::write("./out/licenses.json", licenses_json)?;

    if skipped_files > 0 {
        eprintln!("Warning: skipped {skipped_files} license file(s) due to read/parse issues");
    }

    Ok(())
}
