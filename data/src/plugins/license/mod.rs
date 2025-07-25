use std::collections::HashSet;

use data_lib::license::{LicenseData, LicenseDescriptionNested, Licenses};
use serde_yaml;

pub mod license_compare;

pub fn process_licenses() {
    let dir = std::fs::read_dir("../choosealicense.com/_licenses")
        .expect("Failed to read licenses directory");
    let licenses: Vec<LicenseData> = dir
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_file() {
                    // Bubbling error? Should it be caught?
                    let data = std::fs::read_to_string(&path).ok()?;
                    let parts: Vec<&str> = data.split("---").collect();
                    if parts.len() < 2 {
                        return None;
                    }
                    // Ignoring the error here?
                    serde_yaml::from_str(parts[1]).ok()
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

    let rules_data = std::fs::read_to_string("../choosealicense.com/_data/rules.yml")
        .expect("Failed to read rules data");
    let descriptions: LicenseDescriptionNested =
        serde_yaml::from_str(&rules_data).expect("Failed to parse rules data");

    let licenses_data = Licenses {
        licenses,
        permissions: all_permissions.into_iter().collect(),
        conditions: all_conditions.into_iter().collect(),
        limitations: all_limitations.into_iter().collect(),
        descriptions,
    };

    let licenses_json = serde_json::to_string_pretty(&licenses_data)
        .expect("Failed to serialize licenses data to JSON");
    std::fs::write("./out/licenses.json", licenses_json)
        .expect("Failed to write licenses data to file");
}
