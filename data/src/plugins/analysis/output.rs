use data_lib::plugin::PluginRepoData;

use super::mainjs::MainJsResult;
use crate::plugins::release_acquisition::PluginReleaseStateEntry;

pub(super) trait PluginRepoDataExt {
    fn apply_main_js_analysis(&mut self, result: MainJsResult);
    fn apply_release_state(&mut self, state_entry: &PluginReleaseStateEntry);
}

impl PluginRepoDataExt for PluginRepoData {
    fn apply_main_js_analysis(&mut self, result: MainJsResult) {
        self.estimated_target_es_version = result.estimated_target_es_version;
        self.main_js_findings = result.findings;
        self.main_js_disclosures = result.disclosures;
        self.main_js_diagnostics = result.diagnostics;
        self.main_js_is_probably_minified = result.is_probably_minified;
        self.main_js_minification_score = result.minification_score;
    }

    fn apply_release_state(&mut self, state_entry: &PluginReleaseStateEntry) {
        self.latest_release_main_js_size_bytes = state_entry.latest_release_main_js_size_bytes;
        self.latest_release_tag = state_entry.latest_release_tag.clone();
        self.latest_release_published_at = state_entry.latest_release_published_at.clone();
        self.latest_release_fetch_status = state_entry.latest_release_fetch_status.clone();
    }
}
