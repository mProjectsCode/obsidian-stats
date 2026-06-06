use std::fs;

use data_lib::plugin::{
    MainJsApiCapability, MainJsApiDisclosure, MainJsApiEvidence, PluginData,
    PluginRepoAnalysisError, PluginRepoData,
};
use hashbrown::HashMap;

use super::{
    mainjs::analyze_main_js,
    mainjs::api_classifier::{
        ApiCategory, ApiClassificationResult, ApiMatchKind, ApiSeverity, Confidence,
    },
    repo::{analyze_repo, into_plugin_repo_data},
    run_stats::ExtraRunStats,
    types::AnalysisResult,
};
use crate::plugins::{
    license::license_compare::LicenseComparer,
    release_acquisition::{
        PluginReleaseState, PluginReleaseStateEntry, release_main_js_cache_path,
    },
    stats_helper::HelperPluginStore,
};

const MAX_MAIN_JS_ANALYSIS_BYTES: u64 = 10 * 1024 * 1024;

pub(crate) fn analyze_plugin(
    plugin: &PluginData,
    license_comparer: &LicenseComparer,
    release_state: &PluginReleaseState,
    helper_store: &HelperPluginStore,
    run_stats: &mut ExtraRunStats,
) -> Result<PluginRepoData, String> {
    let repo_result = analyze_repo(plugin, license_comparer).map_err(|error| error.to_string())?;
    let mut output = into_plugin_repo_data(repo_result);
    output.manifest = helper_store.helper_manifest_for_plugin(plugin);
    let mut analysis_result = AnalysisResult::default();

    let Some(state_entry) = matching_release_state_entry(plugin, release_state) else {
        run_stats.release_state_missing += 1;
        return Ok(output);
    };

    apply_release_state_fields(&mut output, state_entry);
    increment_release_status_count(run_stats, state_entry);

    if let Some(tag) = state_entry.latest_release_tag.as_deref() {
        let path = release_main_js_cache_path(&plugin.id, tag);
        if let Ok(path) = path {
            let too_large = fs::metadata(&path)
                .map(|metadata| metadata.len() > MAX_MAIN_JS_ANALYSIS_BYTES)
                .unwrap_or(false);
            if too_large {
                output
                    .analysis_errors
                    .push(PluginRepoAnalysisError::MainJsAnalysisTooLarge);
                run_stats.release_main_js_scan_failed += 1;
            } else if let Ok(bytes) = fs::read(path) {
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
    } else if output.estimated_target_es_version.is_none() {
        run_stats.release_main_js_scan_failed += 1;
    }

    if output.estimated_target_es_version.is_none() {
        output.estimated_target_es_version = analysis_result.mainjs.estimated_target_es_version;
    }

    Ok(output)
}

fn apply_mainjs_fields(output: &mut PluginRepoData, result: &AnalysisResult) {
    output.main_js_parse_succeeded = result.mainjs.parse_succeeded;
    output.main_js_tolerant_parse_required = result.mainjs.tolerant_parse_required;
    output.main_js_is_probably_minified = result.mainjs.is_probably_minified;
    output.main_js_minification_score = result.mainjs.minification_score;
    output.main_js_includes_sourcemap_comment = result.mainjs.includes_sourcemap_comment;
    output.main_js_includes_inline_sourcemap = result.mainjs.includes_inline_sourcemap;
    output.main_js_large_base64_blob_count = result.mainjs.large_base64_blob_count;
    output.main_js_largest_base64_blob_length = result.mainjs.largest_base64_blob_length;
    output.main_js_embedded_blob_type_counts =
        btree_to_hashmap(&result.mainjs.embedded_blob_type_counts);
    output.main_js_worker_usage_count = result.mainjs.worker_usage_count;
    output.main_js_webassembly_usage_count = result.mainjs.webassembly_usage_count;
    output.main_js_dynamic_import_usage_count = result.mainjs.dynamic_import_usage_count;
    output.main_js_bundler_fingerprints = result.mainjs.bundler_fingerprints.clone();
    output.main_js_module_system_fingerprints = result.mainjs.module_system_fingerprints.clone();
    output.main_js_size_bucket = result.mainjs.size_bucket.clone();
    output.main_js_line_count_bucket = result.mainjs.line_count_bucket.clone();
    output.main_js_uses_optional_chaining = result.mainjs.uses_optional_chaining;
    output.main_js_uses_nullish_coalescing = result.mainjs.uses_nullish_coalescing;
    output.main_js_uses_private_fields = result.mainjs.uses_private_fields;
    output.main_js_uses_top_level_await = result.mainjs.uses_top_level_await;
    output.main_js_known_api_host_counts = btree_to_hashmap(&result.mainjs.known_api_host_counts);
    output.main_js_embedded_dependency_name_counts =
        btree_to_hashmap(&result.mainjs.embedded_dependency_name_counts);
    output.main_js_license_banner_count = result.mainjs.license_banner_count;
    output.main_js_credential_literal_count = result.mainjs.credential_literal_count;
    apply_mainjs_api_usage(output, &result.mainjs.api_usage);

    if output.estimated_target_es_version.is_none() {
        output.estimated_target_es_version = result.mainjs.estimated_target_es_version.clone();
    }
}

fn btree_to_hashmap(map: &std::collections::BTreeMap<String, u32>) -> HashMap<String, u32> {
    map.iter()
        .map(|(key, value)| (key.clone(), *value))
        .collect()
}

fn apply_mainjs_api_usage(output: &mut PluginRepoData, api_usage: &ApiClassificationResult) {
    let public_capability_ids = api_usage
        .capabilities()
        .iter()
        .filter(|capability| {
            is_public_mainjs_api_capability(
                capability.severity(),
                capability.confidence(),
            )
        })
        .map(|capability| capability.id().to_string())
        .collect::<std::collections::HashSet<_>>();

    output.main_js_api_capabilities = api_usage
        .capabilities()
        .iter()
        .filter(|capability| public_capability_ids.contains(capability.id()))
        .map(|capability| MainJsApiCapability {
            id: capability.id().to_string(),
            label: capability.label().to_string(),
            category: api_category_name(capability.category()).to_string(),
            severity: api_severity_name(capability.severity()).to_string(),
            confidence: confidence_name(capability.confidence()).to_string(),
            evidence: capability
                .evidence()
                .iter()
                .map(|evidence| MainJsApiEvidence {
                    kind: api_match_kind_name(evidence.kind()).to_string(),
                    symbol: evidence.symbol().to_string(),
                    count: evidence.count(),
                })
                .collect(),
        })
        .collect();
    output.main_js_api_disclosures = api_usage
        .disclosures()
        .iter()
        .filter(|disclosure| public_capability_ids.contains(disclosure.source_capability()))
        .map(|disclosure| MainJsApiDisclosure {
            id: disclosure.id().to_string(),
            from_capability: disclosure.source_capability().to_string(),
        })
        .collect();
}

fn is_public_mainjs_api_capability(
    severity: ApiSeverity,
    confidence: Confidence,
) -> bool {
    if confidence == Confidence::Low {
        return false;
    }

    match severity {
        ApiSeverity::Critical | ApiSeverity::Warning => true,
        _ => confidence == Confidence::High,
    }
}

fn api_category_name(category: ApiCategory) -> &'static str {
    match category {
        ApiCategory::Network => "network",
        ApiCategory::Vault => "vault",
        ApiCategory::Metadata => "metadata",
        ApiCategory::Workspace => "workspace",
        ApiCategory::Editor => "editor",
        ApiCategory::Ui => "ui",
        ApiCategory::Settings => "settings",
        ApiCategory::Lifecycle => "lifecycle",
        ApiCategory::Filesystem => "filesystem",
        ApiCategory::Electron => "electron",
        ApiCategory::Browser => "browser",
        ApiCategory::Dependency => "dependency",
        ApiCategory::DynamicCode => "dynamic_code",
        ApiCategory::Correlation => "correlation",
    }
}

fn api_severity_name(severity: ApiSeverity) -> &'static str {
    match severity {
        ApiSeverity::Info => "info",
        ApiSeverity::Notice => "notice",
        ApiSeverity::Warning => "warning",
        ApiSeverity::Critical => "critical",
    }
}

fn confidence_name(confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::High => "high",
        Confidence::Medium => "medium",
        Confidence::Low => "low",
    }
}

fn api_match_kind_name(kind: ApiMatchKind) -> &'static str {
    match kind {
        ApiMatchKind::Call => "call",
        ApiMatchKind::MemberCall => "member_call",
        ApiMatchKind::MemberRead => "member_read",
        ApiMatchKind::Import => "import",
        ApiMatchKind::StringLiteral => "string_literal",
        ApiMatchKind::Class => "class",
        ApiMatchKind::Constructor => "constructor",
        ApiMatchKind::CallArgument => "call_argument",
        ApiMatchKind::CustomAst => "custom_ast",
        ApiMatchKind::Correlation => "correlation",
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
