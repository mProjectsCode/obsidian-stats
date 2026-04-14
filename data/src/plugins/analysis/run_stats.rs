use data_lib::plugin::PluginExtraData;
use hashbrown::HashMap;

#[derive(Default)]
pub(super) struct ExtraRunStats {
    pub(super) removed_skipped: usize,
    pub(super) repo_extract_failed: usize,
    pub(super) release_state_missing: usize,
    pub(super) release_main_js_scanned: usize,
    pub(super) release_main_js_scan_failed: usize,
    pub(super) status_counts: HashMap<String, usize>,
}

impl ExtraRunStats {
    pub(super) fn merge(&mut self, other: Self) {
        self.removed_skipped += other.removed_skipped;
        self.repo_extract_failed += other.repo_extract_failed;
        self.release_state_missing += other.release_state_missing;
        self.release_main_js_scanned += other.release_main_js_scanned;
        self.release_main_js_scan_failed += other.release_main_js_scan_failed;

        for (status, count) in other.status_counts {
            *self.status_counts.entry(status).or_insert(0) += count;
        }
    }
}

pub(super) struct ExtraPluginResult {
    pub(super) data: PluginExtraData,
    pub(super) stats: ExtraRunStats,
}
