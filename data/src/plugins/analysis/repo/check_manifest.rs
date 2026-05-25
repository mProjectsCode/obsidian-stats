use std::fs;

use data_lib::plugin::{PluginManifest, PluginRepoAnalysisDetailError};

use super::safe_repo_file_path;

pub(super) fn run(
    repo_path: &str,
    plugin_id: &str,
) -> Result<PluginManifest, PluginRepoAnalysisDetailError> {
    let manifest_path = safe_repo_file_path(repo_path, "manifest.json").map_err(|source| {
        PluginRepoAnalysisDetailError::ManifestRead {
            plugin_id: plugin_id.to_string(),
            source,
        }
    })?;
    let manifest = fs::read_to_string(manifest_path).map_err(|source| {
        PluginRepoAnalysisDetailError::ManifestRead {
            plugin_id: plugin_id.to_string(),
            source,
        }
    })?;

    let manifest = manifest.trim_start_matches('\u{feff}');

    serde_json::from_str(manifest).map_err(|source| PluginRepoAnalysisDetailError::ManifestParse {
        plugin_id: plugin_id.to_string(),
        source,
    })
}
