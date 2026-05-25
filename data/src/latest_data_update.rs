use std::{collections::HashMap, path::Path};

use data_lib::{
    latest_data_update::{
        BuildLatestDataUpdateSummaryInputs, PluginPageCloneFreshness, PluginReleaseStateEntryInput,
        ReleaseStatsStateInput,
        build_latest_data_update_summary as build_latest_data_update_summary_from_inputs,
    },
    plugin::{PluginData, PluginExtraData},
    release::{GithubReleaseInfo, ObsidianReleaseInfo},
    theme::ThemeData,
};
use serde::Deserialize;

use crate::{
    constants::{
        CLONE_STATE_PATH, LATEST_DATA_UPDATE_SUMMARY_PATH, PLUGIN_DATA_PATH,
        PLUGIN_RELEASE_ENRICHMENT_STATE_PATH, PLUGIN_REPO_DATA_PATH, RELEASE_CHANGELOG_PATH,
        RELEASE_GITHUB_INTERPOLATED_PATH, RELEASE_GITHUB_RAW_PATH, RELEASE_STATS_STATE_PATH,
        THEME_DATA_PATH,
    },
    file_utils::read_chunked_data,
    state::{read_json_or_default, write_json_atomic},
};

#[derive(Debug, Clone, Deserialize, Default)]
struct CloneState {
    entries: HashMap<String, PluginPageCloneFreshness>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PluginReleaseState {
    entries: HashMap<String, PluginReleaseStateEntryInput>,
}

pub fn build_latest_data_update_summary() -> Result<(), Box<dyn std::error::Error>> {
    let plugins: Vec<PluginData> = read_chunked_data(Path::new(PLUGIN_DATA_PATH))?;
    let themes: Vec<ThemeData> = read_chunked_data(Path::new(THEME_DATA_PATH))?;
    let repo_analysis: Vec<PluginExtraData> = read_chunked_data(Path::new(PLUGIN_REPO_DATA_PATH))?;
    let changelog_releases: Vec<ObsidianReleaseInfo> =
        read_chunked_data(Path::new(RELEASE_CHANGELOG_PATH))?;
    let github_releases: Vec<GithubReleaseInfo> =
        read_chunked_data(Path::new(RELEASE_GITHUB_RAW_PATH))?;
    let interpolated_releases: Vec<GithubReleaseInfo> =
        read_chunked_data(Path::new(RELEASE_GITHUB_INTERPOLATED_PATH))?;

    let clone_state: CloneState = read_json_or_default(Path::new(CLONE_STATE_PATH));
    let release_state: PluginReleaseState =
        read_json_or_default(Path::new(PLUGIN_RELEASE_ENRICHMENT_STATE_PATH));
    let release_stats_state: ReleaseStatsStateInput =
        read_json_or_default(Path::new(RELEASE_STATS_STATE_PATH));

    let release_entries = release_state.entries.into_values().collect::<Vec<_>>();

    let summary =
        build_latest_data_update_summary_from_inputs(BuildLatestDataUpdateSummaryInputs {
            plugins: &plugins,
            themes: &themes,
            repo_analysis_entries: &repo_analysis,
            changelog_releases: &changelog_releases,
            github_releases: &github_releases,
            interpolated_releases: &interpolated_releases,
            clone_entries: &clone_state.entries,
            release_entries: &release_entries,
            release_stats_state: &release_stats_state,
        });

    write_json_atomic(Path::new(LATEST_DATA_UPDATE_SUMMARY_PATH), &summary)?;

    println!(
        "Latest data update summary written to {}",
        LATEST_DATA_UPDATE_SUMMARY_PATH
    );

    Ok(())
}
