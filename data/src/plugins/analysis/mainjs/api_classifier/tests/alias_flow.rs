use super::*;

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
            .severity(ApiSeverity::Info)
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
            .category(ApiCategory::Network)
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
