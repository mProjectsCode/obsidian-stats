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

    let symbols_started = Instant::now();
    for _ in 0..iterations {
        black_box(super::symbol_index::SymbolIndex::collect(
            Some(black_box(&program)),
            black_box(&aliases),
        ));
    }
    let symbols_elapsed = symbols_started.elapsed();

    let classify_started = Instant::now();
    for _ in 0..iterations {
        black_box(classify_api_usage(
            black_box(&source),
            Some(black_box(&program)),
            black_box(rules),
        ));
    }
    let classify_elapsed = classify_started.elapsed();

    eprintln!(
        "bytes={} parse={parse_elapsed:?} aliases_avg={:?} semantics_avg={:?} symbols_avg={:?} classify_avg={:?}",
        source.len(),
        aliases_elapsed / iterations,
        semantics_elapsed / iterations,
        symbols_elapsed / iterations,
        classify_elapsed / iterations,
    );
}

#[test]
fn builder_rejects_rules_without_a_matcher_or_correlation() {
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
fn builder_allows_mixed_primitive_and_dependency_rules() {
    ApiRule::builder("mixed")
        .label("Mixed")
        .category(ApiCategory::Correlation)
        .severity(ApiSeverity::Warning)
        .confidence(Confidence::Medium)
        .calls(["fetch"])
        .requires_all(["disclosure.network_access"])
        .requires_any(["disclosure.note_content_access"])
        .build()
        .unwrap();
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
        ApiCategory::Correlation,
    ];
    let _severities = [
        ApiSeverity::Info,
        ApiSeverity::Notice,
        ApiSeverity::Warning,
        ApiSeverity::Critical,
    ];
    let _confidences = [Confidence::High, Confidence::Medium, Confidence::Low];

    ApiRule::builder("future.primitive")
        .label("Future primitive")
        .category(ApiCategory::Dependency)
        .severity(ApiSeverity::Critical)
        .confidence(Confidence::Low)
        .global_calls(["fetch"])
        .module_calls("obsidian", ["requestUrl"])
        .module_member_calls("obsidian", ["requestUrl"])
        .imports(["obsidian"])
        .string_literals(["obsidian://"])
        .classes(["Notice"])
        .evidence_limit(2)
        .build()
        .unwrap();

    ApiRule::builder("future.correlation")
        .label("Future correlation")
        .category(ApiCategory::Correlation)
        .severity(ApiSeverity::Warning)
        .confidence(Confidence::Medium)
        .when_any(["disclosure.network_access"])
        .build()
        .unwrap();
}

#[test]
fn min_distinct_evidence_requires_multiple_independent_matches() {
    let source = r#"
            fetch("https://example.com/a");
            fetch("https://example.com/b");
        "#;
    let program = parse_program(source);
    let rule = ApiRule::builder("network.needs_two_symbols")
        .label("Needs two evidence symbols")
        .category(ApiCategory::Network)
        .severity(ApiSeverity::Notice)
        .confidence(Confidence::Medium)
        .global_calls(["fetch"])
        .string_literals(["api.example.com"])
        .min_distinct_evidence(2)
        .build()
        .unwrap();
    let result = classify_api_usage(source, program.as_ref(), &[rule]);

    assert!(!result.has_capability("network.needs_two_symbols"));
}

#[test]
fn empty_catalog_emits_no_capabilities_or_disclosures() {
    let source = r#"
            import { requestUrl } from "obsidian";
            requestUrl("https://example.com");
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), &[]);

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
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

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
            .implies(["disclosure.network_access"])
            .build()
            .unwrap(),
        ApiRule::builder("correlation.test")
            .label("Unknown disclosure")
            .category(ApiCategory::Correlation)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::Medium)
            .when_all(["disclosure.not_registered"])
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

#[test]
fn catalog_validation_rejects_unknown_rule_ids() {
    let rules = vec![
        ApiRule::builder("network.test")
            .label("Test network")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .global_calls(["fetch"])
            .implies(["disclosure.network_access"])
            .build()
            .unwrap(),
        ApiRule::builder("correlation.test")
            .label("Unknown rule")
            .category(ApiCategory::Correlation)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::Medium)
            .when_all(["missing.rule"])
            .build()
            .unwrap(),
    ];

    assert_eq!(
        validate_catalog(&rules),
        Err(ApiCatalogError::UnknownRule("missing.rule".to_string()))
    );
}

#[test]
fn built_in_network_rule_detects_common_network_apis() {
    let source = r#"
            import { requestUrl } from "obsidian";
            fetch("https://example.com");
            requestUrl("https://example.com");
            navigator.sendBeacon("https://example.com", "{}");
            new XMLHttpRequest();
            new WebSocket("wss://example.com");
            new EventSource("https://example.com/events");
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("network.browser"));
    assert!(result.has_capability("network.obsidian"));
    assert!(result.has_disclosure("disclosure.network_access"));
    assert!(result.has_disclosure("disclosure.cors_free_network_access"));

    let network = result
        .capabilities
        .iter()
        .find(|capability| capability.id == "network.browser")
        .unwrap();
    assert_eq!(network.evidence.len(), 5);
}

#[test]
fn string_literal_markers_match_inside_larger_literals() {
    let source = r#"
            const callback = "obsidian://open?vault=demo";
            const config = ".obsidian/plugins/example/data.json";
            const endpoint = "https://api.openai.com/v1/chat/completions";
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("vault.uri"));
    assert!(result.has_capability("vault.obsidian_config"));
    assert!(result.has_capability("network.ai_provider"));
    assert!(result.has_disclosure("disclosure.obsidian_config_access"));
    assert!(result.has_disclosure("disclosure.third_party_services"));
}

#[test]
fn parsed_string_markers_ignore_comments() {
    let source = r#"
            // api.openai.com should not classify a provider by itself.
            /* .obsidian/plugins/example */
            const endpoint = getEndpoint();
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("network.ai_provider"));
    assert!(!result.has_capability("vault.obsidian_config"));
}

#[test]
fn private_network_rule_ignores_version_like_literals() {
    let source = r#"
            const version = "10.4.2";
            const range = "172.20.1";
            const text = "192.168.";
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("network.private"));
    assert!(!result.has_disclosure("disclosure.private_network_access"));
}

#[test]
fn dynamic_code_rule_detects_connected_remote_script_dom_injection() {
    let source = r#"
            const script = document.createElement("script");
            script.src = "https://cdn.example.com/plugin.js";
            document.head.appendChild(script);
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("dynamic_code"));
    assert!(result.has_disclosure("disclosure.dynamic_code_or_remote_code"));
}

#[test]
fn dynamic_code_rule_detects_nonliteral_and_inline_script_injection() {
    for source in [
        r#"
            const script = document.createElement("script");
            script.src = getPluginUrl();
            document.head.append(script);
        "#,
        r#"
            const script = document.createElement("script");
            script.textContent = generatedCode;
            document.body.prepend(script);
        "#,
        r#"
            const script = document.createElement("script");
            script.setAttribute("src", relativeUrl);
            document.documentElement.insertBefore(script, document.body);
        "#,
        r#"
            const script = document.createElement("script");
            script.append(generatedCode);
            document.head.appendChild(script);
        "#,
    ] {
        let program = parse_program(source);
        let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

        assert!(
            result.has_capability("dynamic_code"),
            "missed connected script injection in {source}"
        );
    }
}

#[test]
fn dynamic_code_rule_detects_function_constructor_variants() {
    for source in [
        r#"Function("return 1")();"#,
        r#"const F = Function; F("return 1")();"#,
        r#"(function () {}).constructor("return 1")();"#,
        r#"const AsyncFunction = async function () {}.constructor; new AsyncFunction("return 1");"#,
        r#"const GeneratorFunction = (function* () {}).constructor; GeneratorFunction("yield 1");"#,
        r#"const AsyncGeneratorFunction = (async function* () {}).constructor; new AsyncGeneratorFunction("yield 1");"#,
    ] {
        let program = parse_program(source);
        let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

        assert!(
            result.has_capability("dynamic_code"),
            "missed dynamic function constructor in {source}"
        );
    }
}

#[test]
fn dynamic_code_rule_detects_eval_aliases_and_member_forms() {
    for source in [
        r#"const run = eval; run("code");"#,
        r#"(0, eval)("code");"#,
        r#"eval.call(null, "code");"#,
        r#"const run = eval.bind(globalThis); run("code");"#,
        r#"globalThis.eval("code");"#,
        r#"window["eval"]("code");"#,
    ] {
        let program = parse_program(source);
        let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

        assert!(
            result.has_capability("dynamic_code"),
            "missed eval form in {source}"
        );
    }
}

#[test]
fn dynamic_code_rule_detects_string_timers() {
    for source in [
        r#"setTimeout("runCode()", 0);"#,
        r#"window.setInterval(`runCode()`, 1000);"#,
    ] {
        let program = parse_program(source);
        let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

        assert!(result.has_capability("dynamic_code"));
    }
}

#[test]
fn dynamic_code_semantics_respect_shadowing_reassignment_and_callbacks() {
    let source = r#"
        function localOnly(eval, Function, setTimeout) {
            eval("text");
            Function("text");
            Function.constructor("text");
            setTimeout("text", 0);
        }
        let run = globalThis.eval;
        run = safeParser;
        run("text");
        setTimeout(() => runCode(), 0);
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("dynamic_code"));
}

#[test]
fn dynamic_code_rule_ignores_unappended_remote_script_dom_construction() {
    let source = r#"
            const script = document.createElement("script");
            script.src = "https://cdn.example.com/plugin.js";
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("dynamic_code"));
}

#[test]
fn dynamic_code_flow_does_not_cross_sibling_function_bindings() {
    let source = r#"
            function configure() {
                const script = document.createElement("script");
                script.src = "https://cdn.example.com/plugin.js";
            }
            function appendUnrelated() {
                const script = document.createElement("div");
                document.head.appendChild(script);
            }
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("dynamic_code"));
}

#[test]
fn catalog_avoids_reviewed_coarse_disclosures() {
    let source = r#"
            import { clipboard } from "electron";
            const input = document.createElement("input");
            input.type = "text";
            const localModel = { tags: [], links: [], embeds: [] };
            this.app.vault.getRoot();
            const adapter = this.app.vault;
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("ui.file_dialog"));
    assert!(!result.has_capability("metadata.extraction"));
    assert!(!result.has_disclosure("disclosure.metadata_access"));
    assert!(!result.has_capability("vault.enumerate"));
    assert!(!result.has_disclosure("disclosure.full_vault_access"));
    assert!(result.has_capability("electron.desktop"));
    assert!(!result.has_disclosure("disclosure.process_or_shell_access"));
}

#[test]
fn add_event_listener_requires_static_keyboard_or_clipboard_event_argument() {
    let matching = r#"document.addEventListener("keydown", () => {});"#;
    let matching_program = parse_program(matching);
    let matching_result =
        classify_api_usage(matching, matching_program.as_ref(), obsidian_api_rules());
    assert!(matching_result.has_capability("browser.broad_input_hooks"));

    let dynamic = r#"
            const eventName = "keydown";
            document.addEventListener(eventName, () => {});
        "#;
    let dynamic_program = parse_program(dynamic);
    let dynamic_result =
        classify_api_usage(dynamic, dynamic_program.as_ref(), obsidian_api_rules());
    assert!(!dynamic_result.has_capability("browser.broad_input_hooks"));

    let unrelated = r#"
            const key = "keydown";
            document.addEventListener("click", () => {});
        "#;
    let unrelated_program = parse_program(unrelated);
    let unrelated_result =
        classify_api_usage(unrelated, unrelated_program.as_ref(), obsidian_api_rules());
    assert!(!unrelated_result.has_capability("browser.broad_input_hooks"));
}

#[test]
fn argument_constrained_rules_are_collected_independently() {
    let source = r#"
            target.on("alpha", () => {});
            target.on("beta", () => {});
        "#;
    let program = parse_program(source);
    let rules = ["alpha", "beta"].map(|event| {
        ApiRule::builder(format!("event.{event}"))
            .label(format!("Handles {event}"))
            .category(ApiCategory::Browser)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_call("target.on")
            .arg_string(0, [event])
            .build()
            .unwrap()
    });

    let result = classify_api_usage(source, program.as_ref(), &rules);

    assert!(result.has_capability("event.alpha"));
    assert!(result.has_capability("event.beta"));
}

#[test]
fn adapter_alias_operation_matches_but_adapter_reference_alone_does_not_disclose() {
    let reference = r#"const adapter = this.app.vault.adapter;"#;
    let reference_program = parse_program(reference);
    let reference_result =
        classify_api_usage(reference, reference_program.as_ref(), obsidian_api_rules());
    assert!(!reference_result.has_capability("vault.adapter"));
    assert!(!reference_result.has_disclosure("disclosure.adapter_file_access"));

    let operation = r#"
            const adapter = this.app.vault.adapter;
            await adapter.read("daily.md");
        "#;
    let operation_program = parse_program(operation);
    let operation_result =
        classify_api_usage(operation, operation_program.as_ref(), obsidian_api_rules());
    assert!(operation_result.has_capability("vault.adapter"));
    assert!(operation_result.has_disclosure("disclosure.adapter_file_access"));
}

#[test]
fn adapter_flow_does_not_cross_sibling_function_bindings() {
    let source = r#"
            function captureAdapter() {
                const adapter = this.app.vault.adapter;
            }
            function useUnrelated() {
                const adapter = localStorage;
                adapter.read("daily.md");
            }
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("vault.adapter"));
}

#[test]
fn metadata_extraction_requires_metadata_cache_flow() {
    let local = r#"const localModel = { tags: [], links: [] }; localModel.tags;"#;
    let local_program = parse_program(local);
    let local_result = classify_api_usage(local, local_program.as_ref(), obsidian_api_rules());
    assert!(!local_result.has_capability("metadata.extraction"));

    let cache = r#"
            const cache = this.app.metadataCache.getFileCache(file);
            cache.tags;
            cache.links;
        "#;
    let cache_program = parse_program(cache);
    let cache_result = classify_api_usage(cache, cache_program.as_ref(), obsidian_api_rules());
    assert!(cache_result.has_capability("metadata.extraction"));
    assert!(cache_result.has_disclosure("disclosure.metadata_access"));
}

#[test]
fn metadata_flow_does_not_cross_sibling_function_bindings() {
    let source = r#"
            function captureCache() {
                const cache = this.app.metadataCache.getFileCache(file);
            }
            function useUnrelated() {
                const cache = localModel;
                cache.tags;
            }
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("metadata.extraction"));
}

#[test]
fn dom_file_input_requires_connected_input_type_flow() {
    let text = r#"
            const input = document.createElement("input");
            input.type = "text";
        "#;
    let text_program = parse_program(text);
    let text_result = classify_api_usage(text, text_program.as_ref(), obsidian_api_rules());
    assert!(!text_result.has_capability("ui.file_dialog"));

    let file = r#"
            const input = document.createElement("input");
            input.type = "file";
        "#;
    let file_program = parse_program(file);
    let file_result = classify_api_usage(file, file_program.as_ref(), obsidian_api_rules());
    assert!(file_result.has_capability("ui.file_dialog"));
}

#[test]
fn built_in_rules_detect_representative_obsidian_capability_groups() {
    let source = r#"
            import { requestUrl, MarkdownRenderer } from "obsidian";
            class Plugin {
                async onload() {
                    this.addCommand({ id: "x", callback: () => {} });
                    this.registerEditorExtension([]);
                    this.registerMarkdownPostProcessor(() => {});
                    this.registerInterval(setInterval(() => {}, 1000));
                    await requestUrl("https://example.com");
                    const file = this.app.workspace.getActiveFile();
                    const text = await this.app.vault.read(file);
                    this.app.vault.getMarkdownFiles();
                    const cache = this.app.metadataCache.getFileCache(file);
                    cache.frontmatter;
                    MarkdownRenderer.render(this.app, text, this.containerEl, "", this);
                }
            }
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    for expected in [
        "network.obsidian",
        "vault.read",
        "vault.enumerate",
        "metadata.read",
        "metadata.frontmatter",
        "workspace.active_file",
        "ui.commands",
        "editor.extension",
        "editor.markdown_processing",
        "lifecycle.events",
    ] {
        assert!(
            result.has_capability(expected),
            "missing capability {expected}"
        );
    }

    for expected in [
        "disclosure.note_content_access",
        "disclosure.full_vault_access",
        "disclosure.metadata_access",
    ] {
        assert!(
            result.has_disclosure(expected),
            "missing disclosure {expected}"
        );
    }
}

#[test]
fn local_request_url_function_does_not_count_as_obsidian_network_api() {
    let source = r#"
            function requestUrl(url) {
                return `local:${url}`;
            }
            requestUrl("not-network");
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("network.obsidian"));
}

#[test]
fn minified_local_request_url_function_does_not_count_as_obsidian_network_api() {
    let source = r#"function requestUrl(r){return r}requestUrl("not-network");"#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("network.obsidian"));
}

#[test]
fn shadowed_fetch_does_not_count_as_browser_network_api() {
    let source = r#"
            function fetch(value) {
                return value;
            }
            fetch("not-network");
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("network.browser"));
}

#[test]
fn minified_shadowed_fetch_does_not_count_as_browser_network_api() {
    let source = r#"function fetch(r){return r}fetch("not-network");"#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("network.browser"));
}

#[test]
fn local_binding_only_shadows_global_calls_in_its_lexical_scope() {
    let source = r#"
            function localOnly() {
                const fetch = value => value;
                fetch("not-network");
            }
            fetch("https://example.com");
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    let capability = result
        .capabilities
        .iter()
        .find(|capability| capability.id == "network.browser")
        .unwrap();
    let fetch = capability
        .evidence
        .iter()
        .find(|evidence| evidence.symbol == "fetch")
        .unwrap();
    assert_eq!(fetch.count, 1);
}

#[test]
fn parameter_shadowing_does_not_hide_global_calls_in_sibling_scopes() {
    let source = r#"
            function localOnly(fetch) {
                fetch("not-network");
            }
            function networkCall() {
                fetch("https://example.com");
            }
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("network.browser"));
}

#[test]
fn obsidian_named_import_request_url_counts_as_network_api() {
    let source = r#"
            import { requestUrl as r } from "obsidian";
            r("https://example.com");
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("network.obsidian"));
}

#[test]
fn named_import_shadowing_is_resolved_at_the_call_site() {
    let source = r#"
            import { requestUrl } from "obsidian";
            requestUrl("https://example.com");
            function localOnly(requestUrl) {
                requestUrl("not-network");
            }
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    let capability = result
        .capabilities
        .iter()
        .find(|capability| capability.id == "network.obsidian")
        .unwrap();
    assert_eq!(capability.evidence[0].count, 1);
}

#[test]
fn obsidian_namespace_import_request_url_counts_as_network_api() {
    let source = r#"
            import * as obsidian from "obsidian";
            obsidian.requestUrl("https://example.com");
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("network.obsidian"));
}

#[test]
fn namespace_member_matchers_support_nested_and_computed_chains() {
    let source = r#"
            import * as obsidian from "obsidian";
            if (obsidian . Platform ["isMobile"]) {
                console.log("mobile");
            }
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("platform.branching"));
}

#[test]
fn namespace_import_is_not_available_through_a_shadowing_binding() {
    let source = r#"
            import * as obsidian from "obsidian";
            function localOnly(obsidian) {
                obsidian.requestUrl("not-network");
            }
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("network.obsidian"));
}

#[test]
fn minified_obsidian_require_namespace_counts_as_network_api() {
    let source = r#"var o=require("obsidian");o.requestUrl("https://example.com");"#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("network.obsidian"));
}

#[test]
fn minified_commonjs_requires_count_as_imports() {
    let source = r#"var f=require("fs"),e=__toESM(require("electron"));f.readFileSync("x");"#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("filesystem.node"));
    assert!(result.has_capability("electron.desktop"));
}

#[test]
fn member_matcher_definitions_ignore_whitespace_around_chain_segments() {
    let source = r#"this.app.vault.read(file);"#;
    let program = parse_program(source);
    let rule = ApiRule::builder("test.whitespace")
        .label("Whitespace")
        .category(ApiCategory::Vault)
        .severity(ApiSeverity::Info)
        .confidence(Confidence::High)
        .member_calls([" this . app . vault . read "])
        .build()
        .unwrap();
    let result = classify_api_usage(source, program.as_ref(), &[rule]);

    assert!(result.has_capability("test.whitespace"));
}

#[test]
fn rooted_member_matchers_follow_minified_aliases_and_destructuring() {
    let source = r#"
            const a = this.app.vault;
            a.read(file);
            const { vault: v } = this.app;
            v.modify(file, text);
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("vault.read"));
    assert!(result.has_capability("vault.write"));
}

#[test]
fn rooted_member_matchers_cover_remaining_obsidian_api_groups() {
    let source = r#"
        const vault = this.app.vault;
        vault.createFolder("folder");
        vault.getResourcePath(file);

        const workspace = this.app.workspace;
        workspace.getActiveFile();
        workspace.requestSaveLayout();

        const cache = this.app.metadataCache;
        cache.getFileCache(file);
        cache.on("changed", () => {});

        const plugins = this.app.plugins;
        plugins.getPlugin("dataview");
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    for expected in [
        "vault.folder_ops",
        "vault.resources",
        "workspace.active_file",
        "workspace.layout_persistence",
        "metadata.read",
        "metadata.events",
        "plugins.internal_access",
    ] {
        assert!(
            result.has_capability(expected),
            "missing aliased capability {expected}"
        );
    }
}

#[test]
fn rooted_member_matchers_reject_local_api_lookalikes() {
    let source = r#"
            function localOnly() {
                const app = {
                    vault: {
                        read() {},
                        modify() {}
                    }
                };
                app.vault.read(file);
                app.vault.modify(file, text);
            }
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("vault.read"));
    assert!(!result.has_capability("vault.write"));
}

#[test]
fn minified_obsidian_require_destructuring_counts_as_network_api() {
    let source = r#"var{requestUrl:r}=require("obsidian");r("https://example.com");"#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("network.obsidian"));
}

#[test]
fn bundled_obsidian_require_wrapper_counts_as_network_api() {
    let source = r#"var o=__toESM(require("obsidian"));o.requestUrl("https://example.com");"#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("network.obsidian"));
}

#[test]
fn member_calls_are_not_also_member_reads() {
    let source = r#"
            navigator.sendBeacon("https://example.com", "{}");
            const sendBeacon = navigator.sendBeacon;
        "#;
    let program = parse_program(source);
    let rule = ApiRule::builder("test.member_read")
        .label("Reads a member")
        .category(ApiCategory::Browser)
        .severity(ApiSeverity::Info)
        .confidence(Confidence::High)
        .member_reads(["navigator.sendBeacon"])
        .build()
        .unwrap();
    let result = classify_api_usage(source, program.as_ref(), &[rule]);

    let capability = result.capabilities.first().unwrap();
    assert_eq!(capability.evidence[0].count, 1);
}

#[test]
fn class_matchers_detect_referenced_classes_without_construction() {
    let source = r#"
            import { MarkdownView } from "obsidian";
            if (leaf.view instanceof MarkdownView) {
                leaf.view.editor.getValue();
            }
        "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("editor.markdown_api"));
}

#[test]
fn unconstrained_calls_still_match_raw_call_names() {
    let source = r#"
            function customThing() {}
            customThing();
        "#;
    let program = parse_program(source);
    let rule = ApiRule::builder("test.raw_call")
        .label("Raw call")
        .category(ApiCategory::Dependency)
        .severity(ApiSeverity::Info)
        .confidence(Confidence::Low)
        .calls(["customThing"])
        .build()
        .unwrap();
    let result = classify_api_usage(source, program.as_ref(), &[rule]);

    assert!(result.has_capability("test.raw_call"));
}

#[test]
fn primitive_and_correlation_rules_emit_disclosures() {
    let source = r#"
            import { requestUrl } from "obsidian";
            requestUrl("https://example.com");
            this.app.vault.read(file);
        "#;
    let program = parse_program(source);
    let rules = vec![
        ApiRule::builder("network.obsidian_request")
            .label("Uses Obsidian request API")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::High)
            .module_calls("obsidian", ["requestUrl"])
            .implies(["disclosure.network_access"])
            .build()
            .unwrap(),
        ApiRule::builder("vault.read")
            .label("Reads vault files")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_calls(["this.app.vault.read"])
            .implies(["disclosure.note_content_access"])
            .build()
            .unwrap(),
        ApiRule::builder("correlation.vault_read_plus_network")
            .label("Reads vault data and uses network")
            .category(ApiCategory::Correlation)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::Medium)
            .when_all([
                "disclosure.network_access",
                "disclosure.note_content_access",
            ])
            .build()
            .unwrap(),
    ];

    validate_catalog(&rules).unwrap();
    let result = classify_api_usage(source, program.as_ref(), &rules);

    let capability_ids = result
        .capabilities
        .iter()
        .map(|capability| capability.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        capability_ids,
        vec![
            "network.obsidian_request",
            "vault.read",
            "correlation.vault_read_plus_network"
        ]
    );

    let disclosure_ids = result
        .disclosures
        .iter()
        .map(|disclosure| disclosure.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        disclosure_ids,
        vec![
            "disclosure.network_access",
            "disclosure.note_content_access"
        ]
    );
}

#[test]
fn correlations_do_not_depend_on_catalog_order() {
    let source = r#"fetch("https://example.com");"#;
    let program = parse_program(source);
    let rules = vec![
        ApiRule::builder("correlation.network")
            .label("Network correlation")
            .category(ApiCategory::Correlation)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::High)
            .when_all(["network.browser"])
            .build()
            .unwrap(),
        ApiRule::builder("network.browser")
            .label("Browser network")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::High)
            .global_calls(["fetch"])
            .build()
            .unwrap(),
    ];

    let result = classify_api_usage(source, program.as_ref(), &rules);

    assert!(result.has_capability("correlation.network"));
}

#[test]
fn rooted_aliases_follow_later_assignments_nested_destructuring_and_object_properties() {
    let source = r#"
        let late;
        late = this.app.vault;
        late.read(file);

        const { app: { vault: nested } } = this;
        nested.modify(file, text);

        const holder = {};
        holder.vault = this.app.vault;
        holder.vault.getFiles();
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("vault.read"));
    assert!(result.has_capability("vault.write"));
    assert!(result.has_capability("vault.enumerate"));
}

#[test]
fn later_alias_mutation_kills_obsolete_provenance() {
    let source = r#"
        let vault = this.app.vault;
        vault = localStore;
        vault.read(file);
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("vault.read"));
}

#[test]
fn remote_dom_flow_follows_arguments_into_direct_helpers() {
    let source = r#"
        function appendToHead(node) {
            document.head.appendChild(node);
        }
        const script = document.createElement("script");
        script.src = "https://cdn.example.com/plugin.js";
        appendToHead(script);
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("dynamic_code"));
}

#[test]
fn rooted_api_aliases_follow_direct_function_arguments() {
    let source = r#"
        function readFrom(vault) {
            return vault.read(file);
        }
        readFrom(this.app.vault);
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("vault.read"));
}

#[test]
fn semantic_flow_respects_reassignment_order() {
    let source = r#"
        let script = document.createElement("script");
        script.src = "https://cdn.example.com/plugin.js";
        script = document.createElement("div");
        document.head.appendChild(script);
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("dynamic_code"));
}

#[test]
fn semantic_flow_does_not_connect_future_assignments_to_past_uses() {
    let source = r#"
        const script = document.createElement("script");
        document.head.appendChild(script);
        script.src = "https://cdn.example.com/plugin.js";
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("dynamic_code"));
}

#[test]
fn optional_chains_and_static_computed_properties_are_canonicalized() {
    let source = r#"
        this?.app?.vault?.["re" + "ad"]?.(file);
        import * as obsidian from "obsidian";
        obsidian?.Platform?.[`isMobile`];
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("vault.read"));
    assert!(result.has_capability("platform.branching"));
}

#[test]
fn correlations_require_evidence_in_the_same_function() {
    let rules = vec![
        ApiRule::builder("network")
            .label("Network")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::High)
            .global_calls(["fetch"])
            .build()
            .unwrap(),
        ApiRule::builder("vault")
            .label("Vault")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .rooted_member_calls(["this.app.vault.read"])
            .build()
            .unwrap(),
        ApiRule::builder("combined")
            .label("Combined")
            .category(ApiCategory::Correlation)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::High)
            .when_all(["network", "vault"])
            .build()
            .unwrap(),
    ];
    let unrelated = r#"
        function upload() { fetch("https://example.com"); }
        function read() { this.app.vault.read(file); }
    "#;
    let unrelated_program = parse_program(unrelated);
    let unrelated_result = classify_api_usage(unrelated, unrelated_program.as_ref(), &rules);
    assert!(!unrelated_result.has_capability("combined"));

    let related = r#"
        function sync() {
            const text = this.app.vault.read(file);
            fetch("https://example.com", { body: text });
        }
    "#;
    let related_program = parse_program(related);
    let related_result = classify_api_usage(related, related_program.as_ref(), &rules);
    assert!(related_result.has_capability("combined"));
}

#[test]
fn local_classes_and_constructors_do_not_impersonate_obsidian_apis() {
    let source = r#"
        class Notice {}
        class Setting {}
        class MarkdownView {}
        new Notice("local");
        new Setting(container);
        const view = new MarkdownView();
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("ui.modals_notices"));
    assert!(!result.has_capability("settings.ui"));
    assert!(!result.has_capability("editor.markdown_api"));
}

#[test]
fn imported_obsidian_ui_base_classes_count_when_referenced() {
    let source = r#"
        import { Modal, Notice } from "obsidian";
        class ExampleModal extends Modal {}
        const show = () => new Notice("done");
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("ui.modals_notices"));
}

#[test]
fn lifecycle_rule_detects_class_method_declarations() {
    let source = r#"
        class ExamplePlugin {
            async onload() {}
            onunload() {}
        }
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("lifecycle.methods"));
}

#[test]
fn unused_obsidian_class_imports_are_not_class_usage() {
    let source = r#"
        import { Notice, Setting, MarkdownView } from "obsidian";
        console.log("imports only");
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("ui.modals_notices"));
    assert!(!result.has_capability("settings.ui"));
    assert!(!result.has_capability("editor.markdown_api"));
}

#[test]
fn broad_provider_and_header_words_do_not_match_from_prose() {
    let source = r#"
        const docs = "mastodon posthog headers";
    "#;
    let program = parse_program(source);
    let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("network.sync_storage_provider"));
    assert!(!result.has_capability("network.telemetry"));
    assert!(!result.has_capability("network.headers"));
}

#[test]
fn parse_failure_does_not_use_raw_substring_classification() {
    let source = r#"
        // fetch("https://example.com")
        const prose = "this.app.vault.read headers posthog";
        function broken( {
    "#;
    assert!(parse_program(source).is_none());
    let result = classify_api_usage(source, None, obsidian_api_rules());

    assert!(result.capabilities.is_empty());
    assert!(result.disclosures.is_empty());
}
