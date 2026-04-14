use std::{fs, path::Path};

use data_lib::input_data::{ObsCommunityPluginDeprecations, ObsCommunityPluginRemoved};

use crate::constants::{OBS_RELEASES_REPO_PATH, PLUGIN_DEPRECATIONS_PATH, PLUGIN_REMOVED_PATH};

pub(super) fn read_removed_plugins()
-> Result<Vec<ObsCommunityPluginRemoved>, Box<dyn std::error::Error>> {
    let plugin_removed_list = fs::read_to_string(Path::new(&format!(
        "{OBS_RELEASES_REPO_PATH}/{PLUGIN_REMOVED_PATH}",
    )))
    .map_err(|e| format!("Failed to read plugin removed list: {e}"))?;

    Ok(serde_json::from_str(&plugin_removed_list)?)
}

pub(super) fn read_plugin_version_deprecations()
-> Result<ObsCommunityPluginDeprecations, Box<dyn std::error::Error>> {
    let plugin_deprecations = fs::read_to_string(Path::new(&format!(
        "{OBS_RELEASES_REPO_PATH}/{PLUGIN_DEPRECATIONS_PATH}",
    )))
    .map_err(|e| format!("Failed to read plugin deprecations: {e}"))?;

    Ok(serde_json::from_str(&plugin_deprecations)?)
}
