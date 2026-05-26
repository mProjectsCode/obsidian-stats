use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

mod summary;

pub use summary::{BuildLatestDataUpdateSummaryInputs, build_latest_data_update_summary};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReleaseStateEntryInput {
    pub repo: String,
    pub last_checked_unix: i64,
    pub latest_release_main_js_size_bytes: Option<u64>,
    pub latest_release_fetch_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReleaseStatsStateInput {
    pub last_fetch_unix: Option<i64>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct CountShare {
    pub label: String,
    pub count: usize,
    pub share: f64,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct LatestDataUpdateSummary {
    pub refreshed_at_unix: Option<i64>,
    pub clone_run_at_unix: Option<i64>,
    pub release_run_at_unix: Option<i64>,
    pub obsidian_release_fetch_at_unix: Option<i64>,
    pub latest_plugin_download_snapshot_date: Option<String>,
    pub latest_obsidian_release_date: Option<String>,
    pub latest_obsidian_version: Option<String>,
    pub plugins: PluginSummary,
    pub themes: ThemeSummary,
    pub releases: ReleaseSummary,
    pub clone: CloneSummary,
    pub release_acquisition: ReleaseAcquisitionSummary,
    pub repo_analysis: RepoAnalysisSummary,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct PluginSummary {
    pub total: usize,
    pub active: usize,
    pub removed: usize,
    pub total_downloads: u64,
    pub version_snapshots: usize,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct ThemeSummary {
    pub total: usize,
    pub active: usize,
    pub removed: usize,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct ReleaseSummary {
    pub changelog_entries: usize,
    pub github_release_snapshots: usize,
    pub interpolated_release_snapshots: usize,
    pub asset_count: usize,
    pub download_snapshots: usize,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct CloneSummary {
    pub tracked: usize,
    pub ok: usize,
    pub skipped: usize,
    pub failed: usize,
    pub success_rate: f64,
    pub failed_plugins: Vec<String>,
    pub status_counts: Vec<CountShare>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct LargestMainJs {
    pub repo: Option<String>,
    pub size_bytes: u64,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct ReleaseAcquisitionSummary {
    pub tracked: usize,
    pub ok: usize,
    pub retained_previous_main_js: usize,
    pub no_release: usize,
    pub no_main_js_asset: usize,
    pub error_count: usize,
    pub main_js_coverage: usize,
    pub main_js_coverage_rate: f64,
    pub total_main_js_bytes: u64,
    pub average_main_js_bytes: u64,
    pub largest_main_js: LargestMainJs,
    pub status_counts: Vec<CountShare>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct RepoAnalysisSummary {
    pub tracked: usize,
    pub active_success: usize,
    pub active_failures: usize,
    pub removed_skipped: usize,
    pub coverage_rate: f64,
    pub failure_samples: Vec<String>,
    pub error_counts: Vec<CountShare>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct PluginPageCloneFreshness {
    pub repo: String,
    pub target_release_tag: Option<String>,
    pub last_attempt_unix: i64,
    pub last_success_unix: Option<i64>,
    pub status: String,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct PluginPageReleaseFreshness {
    pub repo: String,
    pub last_checked_unix: i64,
    pub latest_release_fetch_status: Option<String>,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct PluginPageFreshness {
    pub summary: LatestDataUpdateSummary,
    pub clone: Option<PluginPageCloneFreshness>,
    pub release: Option<PluginPageReleaseFreshness>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PluginPageCloneState {
    pub entries: HashMap<String, PluginPageCloneFreshness>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PluginPageReleaseState {
    pub entries: HashMap<String, PluginPageReleaseFreshness>,
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct PluginPageFreshnessData {
    #[wasm_bindgen(skip)]
    pub summary: LatestDataUpdateSummary,
    #[wasm_bindgen(skip)]
    pub clone_entries: HashMap<String, PluginPageCloneFreshness>,
    #[wasm_bindgen(skip)]
    pub release_entries: HashMap<String, PluginPageReleaseFreshness>,
}

impl PluginPageFreshnessData {
    pub fn new(
        summary: LatestDataUpdateSummary,
        clone_state: PluginPageCloneState,
        release_state: PluginPageReleaseState,
    ) -> Self {
        Self {
            summary,
            clone_entries: clone_state.entries,
            release_entries: release_state.entries,
        }
    }
}

#[wasm_bindgen]
impl PluginPageFreshnessData {
    pub fn summary(&self) -> LatestDataUpdateSummary {
        self.summary.clone()
    }

    pub fn get(&self, plugin_id: &str) -> PluginPageFreshness {
        PluginPageFreshness {
            summary: self.summary.clone(),
            clone: self.clone_entries.get(plugin_id).cloned(),
            release: self.release_entries.get(plugin_id).cloned(),
        }
    }
}
