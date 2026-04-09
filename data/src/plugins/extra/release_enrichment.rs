use data_lib::plugin::{PluginData, PluginRepoData};

use super::{es_inference::infer_es_from_main_js, run_stats::ExtraRunStats};
use crate::plugins::release_acquisition::{PluginReleaseState, PluginReleaseStateEntry};

pub(super) fn enrich_release_metadata(
    plugin: &PluginData,
    repo_data: &mut PluginRepoData,
    release_state: &PluginReleaseState,
    run_stats: &mut ExtraRunStats,
) {
    let Some(state_entry) = matching_release_state_entry(plugin, release_state) else {
        run_stats.release_state_missing += 1;
        return;
    };

    apply_release_state_fields(repo_data, state_entry);
    increment_release_status_count(run_stats, state_entry);

    if repo_data.estimated_target_es_version.is_some() {
        run_stats.release_main_js_scanned += 1;
        return;
    }

    let estimated_from_main_js = state_entry
        .latest_release_tag
        .as_deref()
        .and_then(|tag| infer_es_from_main_js(&plugin.id, tag));

    if let Some(es) = estimated_from_main_js {
        repo_data.estimated_target_es_version = Some(es);
        run_stats.release_main_js_scanned += 1;
    } else {
        run_stats.release_main_js_scan_failed += 1;
    }
}

fn apply_release_state_fields(
    repo_data: &mut PluginRepoData,
    state_entry: &PluginReleaseStateEntry,
) {
    repo_data.latest_release_main_js_size_bytes = state_entry.latest_release_main_js_size_bytes;
    repo_data.estimated_target_es_version = state_entry.estimated_target_es_version.clone();
    repo_data.latest_release_tag = state_entry.latest_release_tag.clone();
    repo_data.latest_release_published_at = state_entry.latest_release_published_at.clone();
    repo_data.latest_release_fetch_status = state_entry.latest_release_fetch_status.clone();
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
