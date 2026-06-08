use super::*;

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
