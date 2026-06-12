use super::{
    ApiCatalogError, ApiCategory, ApiRule, ApiRuleBuildError, ApiSeverity, Confidence,
    classify_api_usage, obsidian_api_rules, validate_catalog,
};
use crate::plugins::analysis::mainjs::parse_program;

#[test]
#[ignore = "manual performance benchmark"]
fn benchmark_real_main_js() {
    use std::{fs, hint::black_box, time::Instant};

    let path = std::env::var("API_CLASSIFIER_BENCH_PATH")
        .expect("set API_CLASSIFIER_BENCH_PATH to a main.js file");
    let iterations = std::env::var("API_CLASSIFIER_BENCH_ITERATIONS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(10);
    let source = fs::read_to_string(path).unwrap();

    let parse_started = Instant::now();
    let program = parse_program(black_box(&source)).expect("benchmark input should parse");
    let parse_elapsed = parse_started.elapsed();

    let rules = obsidian_api_rules();
    let aliases_started = Instant::now();
    for _ in 0..iterations {
        black_box(super::symbol_index::AliasInfo::collect(black_box(&program)));
    }
    let aliases_elapsed = aliases_started.elapsed();
    let aliases = super::symbol_index::AliasInfo::collect(&program);

    let semantics_started = Instant::now();
    for _ in 0..iterations {
        black_box(super::custom_matchers::SemanticIndex::collect(
            black_box(&program),
            black_box(&aliases),
        ));
    }
    let semantics_elapsed = semantics_started.elapsed();

    let classify_started = Instant::now();
    for _ in 0..iterations {
        black_box(classify_api_usage(
            Some(black_box(&program)),
            black_box(rules),
        ));
    }
    let classify_elapsed = classify_started.elapsed();

    eprintln!(
        "bytes={} parse={parse_elapsed:?} aliases_avg={:?} semantics_avg={:?} classify_avg={:?}",
        source.len(),
        aliases_elapsed / iterations,
        semantics_elapsed / iterations,
        classify_elapsed / iterations,
    );
}

#[test]
fn builder_rejects_rules_without_a_matcher() {
    let error = ApiRule::builder("vault.read")
        .label("Reads vault files")
        .category(ApiCategory::Vault)
        .severity(ApiSeverity::Info)
        .confidence(Confidence::High)
        .build()
        .unwrap_err();

    assert_eq!(error, ApiRuleBuildError::MissingMatcher);
}

#[test]
fn builder_supports_future_matcher_buckets_and_taxonomy_values() {
    let _categories = [
        ApiCategory::Network,
        ApiCategory::Vault,
        ApiCategory::Metadata,
        ApiCategory::Workspace,
        ApiCategory::Editor,
        ApiCategory::Ui,
        ApiCategory::Settings,
        ApiCategory::Lifecycle,
        ApiCategory::Filesystem,
        ApiCategory::Electron,
        ApiCategory::Browser,
        ApiCategory::Dependency,
        ApiCategory::DynamicCode,
    ];
    let _severities = [ApiSeverity::Info, ApiSeverity::Warning];
    let _confidences = [Confidence::High, Confidence::Medium, Confidence::Low];

    ApiRule::builder("future.primitive")
        .label("Future primitive")
        .category(ApiCategory::Dependency)
        .severity(ApiSeverity::Warning)
        .confidence(Confidence::Low)
        .global_calls(["fetch"])
        .module_calls("obsidian", ["requestUrl"])
        .module_member_calls("obsidian", ["requestUrl"])
        .imports(["obsidian"])
        .string_literals(["obsidian://"])
        .classes(["Notice"])
        .build()
        .unwrap();
}

#[test]
fn empty_catalog_emits_no_capabilities_or_disclosures() {
    let source = r#"
            import { requestUrl } from "obsidian";
            requestUrl("https://example.com");
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(program.as_ref(), &[]);

    assert!(result.capabilities.is_empty());
    assert!(result.disclosures.is_empty());
}

#[test]
fn built_in_catalog_is_valid_and_has_stable_core_capability_groups() {
    let rules = obsidian_api_rules();
    validate_catalog(rules).unwrap();

    let rule_ids = rules
        .iter()
        .map(|rule| rule.id.as_str())
        .collect::<Vec<_>>();
    for expected in [
        "network.browser",
        "network.obsidian",
        "vault.read",
        "vault.write",
        "vault.destructive",
        "vault.enumerate",
        "vault.adapter",
        "metadata.read",
        "metadata.frontmatter",
        "workspace.views",
        "workspace.active_file",
        "editor.extension",
        "editor.markdown_processing",
        "ui.commands",
        "ui.modals_notices",
        "settings.persistence",
        "settings.ui",
        "lifecycle.events",
        "plugins.internal_access",
        "platform.branching",
        "filesystem.node",
        "process.node",
        "electron.desktop",
        "browser.storage",
        "browser.permissions",
        "browser.broad_input_hooks",
        "dynamic_code",
        "network.remote_dom_loading",
        "ui.file_dialog",
        "metadata.extraction",
        "workspace.layout_persistence",
    ] {
        assert!(
            rule_ids.contains(&expected),
            "missing built-in rule {expected}"
        );
    }
}

#[test]
fn built_in_rules_detect_remaining_static_risk_groups() {
    let source = r#"
            import { dialog } from "electron";
            import { requestUrl } from "obsidian";
            dialog.showOpenDialog({ properties: ["openFile"] });
            this.app.workspace.requestSaveLayout();
            this.app.plugins.getPlugin("dataview");
            this.app.metadataCache.on("changed", () => {});
            const file = this.app.workspace.getActiveFile();
            const cache = this.app.metadataCache.getFileCache(file);
            const tags = cache.tags;
            const links = cache.links;
            const embeds = cache.embeds;
            document.addEventListener("keydown", () => {});
            const script = document.createElement("script");
            script.src = "https://cdn.example.com/plugin.js";
            document.head.appendChild(script);
            const img = document.createElement("img");
            img.src = "https://cdn.example.com/logo.png";
            document.body.appendChild(img);
            await requestUrl("https://example.com");
            this.app.vault.getFiles();
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(program.as_ref(), obsidian_api_rules());

    for expected in [
        "ui.file_dialog",
        "workspace.layout_persistence",
        "plugins.internal_access",
        "metadata.events",
        "metadata.extraction",
        "browser.broad_input_hooks",
        "network.remote_dom_loading",
        "vault.enumerate",
    ] {
        assert!(
            result.has_capability(expected),
            "missing capability {expected}"
        );
    }

    for expected in [
        "disclosure.workspace_layout",
        "disclosure.plugin_internals",
        "disclosure.metadata_access",
        "disclosure.global_handlers_or_timers",
        "disclosure.network_access",
        "disclosure.full_vault_access",
    ] {
        assert!(
            result.has_disclosure(expected),
            "missing disclosure {expected}"
        );
    }
}

#[test]
fn catalog_validation_rejects_unknown_disclosure_ids() {
    let rules = vec![
        ApiRule::builder("network.test")
            .label("Test network")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .global_calls(["fetch"])
            .implies(["disclosure.not_registered"])
            .build()
            .unwrap(),
    ];

    assert_eq!(
        validate_catalog(&rules),
        Err(ApiCatalogError::UnknownDisclosure(
            "disclosure.not_registered".to_string()
        ))
    );
}

mod alias_flow;
mod capabilities;
mod classes;
mod network;
mod semantic_matchers;
mod symbol_resolution;
