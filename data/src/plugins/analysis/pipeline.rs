use std::path::Path;

use data_lib::plugin::{PluginData, PluginRepoData};

use super::{
    mainjs::analyze_main_js_file, output::PluginRepoDataExt, repo::analyze_repo,
    run_stats::ExtraRunStats,
};
use crate::plugins::{
    license::license_compare::LicenseComparer,
    release_acquisition::{
        PluginReleaseState, PluginReleaseStateEntry, release_main_js_cache_path,
    },
    stats_helper::HelperPluginStore,
};

pub(crate) fn analyze_plugin(
    plugin: &PluginData,
    license_comparer: &LicenseComparer,
    release_state: &PluginReleaseState,
    helper_store: &HelperPluginStore,
    run_stats: &mut ExtraRunStats,
) -> Result<PluginRepoData, String> {
    let repo_result = analyze_repo(plugin, license_comparer).map_err(|error| error.to_string())?;
    let mut output = repo_result.into_plugin_repo_data();
    output.manifest = helper_store.helper_manifest_for_plugin(plugin);

    let Some(state_entry) = matching_release_state_entry(plugin, release_state) else {
        run_stats.release_state_missing += 1;
        return Ok(output);
    };

    output.apply_release_state(state_entry);
    increment_release_status_count(run_stats, state_entry);

    if let Some(tag) = state_entry.latest_release_tag.as_deref() {
        let path = release_main_js_cache_path(&plugin.id, tag);
        if let Ok(path) = path {
            if let Ok(mainjs) = analyze_main_js_file(Path::new(&path)) {
                let failed = mainjs.diagnostics.iter().any(|diagnostic| {
                    matches!(
                        diagnostic.code.as_str(),
                        "source_too_large" | "invalid_utf8"
                    )
                });
                output.apply_main_js_analysis(mainjs);
                if failed {
                    run_stats.release_main_js_scan_failed += 1;
                } else {
                    run_stats.release_main_js_scanned += 1;
                }
            } else {
                run_stats.release_main_js_scan_failed += 1;
            }
        } else {
            run_stats.release_main_js_scan_failed += 1;
        }
    } else {
        run_stats.release_main_js_scan_failed += 1;
    }

    Ok(output)
}

fn matching_release_state_entry<'a>(
    plugin: &PluginData,
    release_state: &'a PluginReleaseState,
) -> Option<&'a PluginReleaseStateEntry> {
    let state_entry = release_state.entries.get(&plugin.id)?;
    if state_entry.repo == plugin.current_entry.repo {
        Some(state_entry)
    } else {
        None
    }
}

fn increment_release_status_count(
    run_stats: &mut ExtraRunStats,
    state_entry: &PluginReleaseStateEntry,
) {
    if let Some(status) = &state_entry.latest_release_fetch_status {
        *run_stats.status_counts.entry(status.clone()).or_insert(0) += 1;
    }
}
