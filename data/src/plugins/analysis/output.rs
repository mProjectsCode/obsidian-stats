use std::collections::HashSet;

use data_lib::plugin::{
    MainJsApiCapability, MainJsApiDisclosure, MainJsApiEvidence, PluginRepoData,
};

use super::{
    mainjs::api_classifier::{ApiClassificationResult, ApiSeverity, Confidence},
    types::MainJsResult,
};
use crate::plugins::release_acquisition::PluginReleaseStateEntry;

pub(super) trait PluginRepoDataExt {
    fn apply_main_js_analysis(&mut self, result: &MainJsResult);
    fn apply_release_state(&mut self, state_entry: &PluginReleaseStateEntry);
}

impl PluginRepoDataExt for PluginRepoData {
    fn apply_main_js_analysis(&mut self, result: &MainJsResult) {
        self.main_js_parse_succeeded = result.parse_succeeded;
        self.main_js_tolerant_parse_required = result.tolerant_parse_required;
        self.main_js_is_probably_minified = result.is_probably_minified;
        self.main_js_minification_score = result.minification_score;
        self.main_js_dynamic_import_usage_count = result.dynamic_import_usage_count;
        self.main_js_bundler_fingerprints = result.bundler_fingerprints.clone();
        self.main_js_module_system_fingerprints = result.module_system_fingerprints.clone();
        self.main_js_size_bucket = result.size_bucket.clone();
        self.main_js_line_count_bucket = result.line_count_bucket.clone();
        self.main_js_uses_optional_chaining = result.uses_optional_chaining;
        self.main_js_uses_nullish_coalescing = result.uses_nullish_coalescing;
        self.main_js_uses_private_fields = result.uses_private_fields;
        self.main_js_uses_top_level_await = result.uses_top_level_await;
        self.main_js_known_api_host_counts = result
            .known_api_host_counts
            .iter()
            .map(|(key, value)| (key.clone(), *value))
            .collect();
        self.main_js_embedded_dependency_name_counts = result
            .embedded_dependency_name_counts
            .iter()
            .map(|(key, value)| (key.clone(), *value))
            .collect();
        self.main_js_license_banner_count = result.license_banner_count;
        self.main_js_credential_literal_count = result.credential_literal_count;
        self.apply_main_js_api_usage(&result.api_usage);

        if self.estimated_target_es_version.is_none() {
            self.estimated_target_es_version = result.estimated_target_es_version.clone();
        }
    }

    fn apply_release_state(&mut self, state_entry: &PluginReleaseStateEntry) {
        self.latest_release_main_js_size_bytes = state_entry.latest_release_main_js_size_bytes;
        self.estimated_target_es_version = state_entry.estimated_target_es_version.clone();
        self.latest_release_tag = state_entry.latest_release_tag.clone();
        self.latest_release_published_at = state_entry.latest_release_published_at.clone();
        self.latest_release_fetch_status = state_entry.latest_release_fetch_status.clone();
    }
}

trait MainJsApiOutputExt {
    fn apply_main_js_api_usage(&mut self, api_usage: &ApiClassificationResult);
}

impl MainJsApiOutputExt for PluginRepoData {
    fn apply_main_js_api_usage(&mut self, api_usage: &ApiClassificationResult) {
        let public_capability_ids = api_usage
            .capabilities()
            .iter()
            .filter(|capability| {
                is_public_capability(capability.severity(), capability.confidence())
            })
            .map(|capability| capability.id().to_string())
            .collect::<HashSet<_>>();

        self.main_js_api_capabilities = api_usage
            .capabilities()
            .iter()
            .filter(|capability| public_capability_ids.contains(capability.id()))
            .map(|capability| MainJsApiCapability {
                id: capability.id().to_string(),
                label: capability.label().to_string(),
                category: capability.category().as_str().to_string(),
                severity: capability.severity().as_str().to_string(),
                confidence: capability.confidence().as_str().to_string(),
                evidence: capability
                    .evidence()
                    .iter()
                    .map(|evidence| MainJsApiEvidence {
                        kind: evidence.kind().as_str().to_string(),
                        symbol: evidence.symbol().to_string(),
                        count: evidence.count(),
                    })
                    .collect(),
            })
            .collect();
        self.main_js_api_disclosures = api_usage
            .disclosures()
            .iter()
            .filter(|disclosure| public_capability_ids.contains(disclosure.source_capability()))
            .map(|disclosure| MainJsApiDisclosure {
                id: disclosure.id().to_string(),
                from_capability: disclosure.source_capability().to_string(),
            })
            .collect();
    }
}

fn is_public_capability(severity: ApiSeverity, confidence: Confidence) -> bool {
    if confidence == Confidence::Low {
        return false;
    }

    match severity {
        ApiSeverity::Warning => true,
        _ => confidence == Confidence::High,
    }
}
