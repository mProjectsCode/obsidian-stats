use std::fs;

use data_lib::plugin::{PluginData, PluginRepoData};

use super::{
    mainjs::analyze_main_js,
    repo::{analyze_repo, into_plugin_repo_data},
    run_stats::ExtraRunStats,
    types::AnalysisResult,
};
use crate::plugins::{
    license::license_compare::LicenseComparer,
    release_acquisition::{
        PluginReleaseState, PluginReleaseStateEntry, release_main_js_cache_path,
    },
};

pub(crate) fn analyze_plugin(
    plugin: &PluginData,
    license_comparer: &LicenseComparer,
    release_state: &PluginReleaseState,
    run_stats: &mut ExtraRunStats,
) -> Result<PluginRepoData, String> {
    let repo_result = analyze_repo(plugin, license_comparer).map_err(|error| error.to_string())?;
    let mut output = into_plugin_repo_data(repo_result);
    let mut analysis_result = AnalysisResult::default();

    let Some(state_entry) = matching_release_state_entry(plugin, release_state) else {
        run_stats.release_state_missing += 1;
        return Ok(output);
    };

    apply_release_state_fields(&mut output, state_entry);
    increment_release_status_count(run_stats, state_entry);

    if let Some(tag) = state_entry.latest_release_tag.as_deref() {
        let path = release_main_js_cache_path(&plugin.id, tag);
        if let Ok(bytes) = fs::read(path) {
            if let Ok(source) = std::str::from_utf8(&bytes) {
                let mainjs = analyze_main_js(source);
                analysis_result.mainjs = mainjs;
                apply_mainjs_fields(&mut output, &analysis_result);
                run_stats.release_main_js_scanned += 1;
            } else {
                run_stats.release_main_js_scan_failed += 1;
            }
        } else if output.estimated_target_es_version.is_none() {
            run_stats.release_main_js_scan_failed += 1;
        }
    } else if output.estimated_target_es_version.is_none() {
        run_stats.release_main_js_scan_failed += 1;
    }

    if output.estimated_target_es_version.is_none() {
        output.estimated_target_es_version = analysis_result.mainjs.estimated_target_es_version;
    }

    Ok(output)
}

fn apply_mainjs_fields(output: &mut PluginRepoData, result: &AnalysisResult) {
    output.main_js_is_probably_minified = result.mainjs.is_probably_minified;
    output.main_js_minification_score = result.mainjs.minification_score;
    output.main_js_includes_sourcemap_comment = result.mainjs.includes_sourcemap_comment;
    output.main_js_large_base64_blob_count = result.mainjs.large_base64_blob_count;
    output.main_js_largest_base64_blob_length = result.mainjs.largest_base64_blob_length;
    output.main_js_worker_usage_count = result.mainjs.worker_usage_count;
    output.main_js_webassembly_usage_count = result.mainjs.webassembly_usage_count;

    if output.estimated_target_es_version.is_none() {
        output.estimated_target_es_version = result.mainjs.estimated_target_es_version.clone();
    }
}

fn apply_release_state_fields(output: &mut PluginRepoData, state_entry: &PluginReleaseStateEntry) {
    output.latest_release_main_js_size_bytes = state_entry.latest_release_main_js_size_bytes;
    output.estimated_target_es_version = state_entry.estimated_target_es_version.clone();
    output.latest_release_tag = state_entry.latest_release_tag.clone();
    output.latest_release_published_at = state_entry.latest_release_published_at.clone();
    output.latest_release_fetch_status = state_entry.latest_release_fetch_status.clone();
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
