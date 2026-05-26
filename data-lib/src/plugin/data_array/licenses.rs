use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    common::increment_named_data_points,
    license::Licenses,
    plugin::{LicenseInfo, PluginLicenseDataPoints},
};

use super::{PluginDataArray, PluginDataArrayView};

#[wasm_bindgen]
impl PluginDataArrayView {
    pub fn license_data_points(
        &self,
        data: &PluginDataArray,
        license_data_string: String,
    ) -> Result<PluginLicenseDataPoints, String> {
        let licenses: Licenses = serde_json::from_str(&license_data_string)
            .map_err(|e| format!("Failed to parse license data: {e}"))?;

        let mut points = PluginLicenseDataPoints {
            licenses: Vec::new(),
            permissions: Vec::new(),
            conditions: Vec::new(),
            limitations: Vec::new(),
            descriptions: licenses.descriptions,
        };

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            match &repo_data.file_license {
                LicenseInfo::Known(name) => {
                    let license_data = licenses.licenses.iter().find(|l| *name == l.spdx_id);

                    if let Some(license_data) = license_data {
                        increment_named_data_points(
                            &mut points.licenses,
                            &license_data.spdx_id,
                            1.0,
                        );

                        for permission in &license_data.permissions {
                            increment_named_data_points(&mut points.permissions, permission, 1.0);
                        }
                        for condition in &license_data.conditions {
                            increment_named_data_points(&mut points.conditions, condition, 1.0);
                        }
                        for limitation in &license_data.limitations {
                            increment_named_data_points(&mut points.limitations, limitation, 1.0);
                        }
                    } else {
                        increment_named_data_points(
                            &mut points.licenses,
                            &LicenseInfo::Unrecognized.to_fancy_string(),
                            1.0,
                        );
                    }
                }
                other => {
                    increment_named_data_points(
                        &mut points.licenses,
                        &other.to_fancy_string(),
                        1.0,
                    );
                }
            }
        });

        Ok(points)
    }
}
