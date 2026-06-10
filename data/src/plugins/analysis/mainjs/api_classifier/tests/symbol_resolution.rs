use super::*;

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
fn bundled_obsidian_request_calls_count_as_network_api() {
    for source in [
        r#"
            import { requestUrl } from "obsidian";
            (0, requestUrl)("https://example.com");
        "#,
        r#"
            import * as obsidian from "obsidian";
            (0, obsidian.request)("https://example.com");
        "#,
        r#"
            const obsidian = require("obsidian");
            (0, obsidian["requestUrl"])("https://example.com");
        "#,
    ] {
        let program = parse_program(source);
        let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

        assert!(
            result.has_capability("network.obsidian"),
            "missed bundled request call in {source}"
        );
    }
}

#[test]
fn obsidian_request_function_aliases_preserve_module_provenance() {
    for source in [
        r#"
            import { requestUrl } from "obsidian";
            const send = requestUrl;
            send("https://example.com");
        "#,
        r#"
            import * as obsidian from "obsidian";
            const send = obsidian.request;
            send("https://example.com");
        "#,
    ] {
        let program = parse_program(source);
        let result = classify_api_usage(source, program.as_ref(), obsidian_api_rules());

        assert!(
            result.has_capability("network.obsidian"),
            "missed aliased request call in {source}"
        );
    }
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
fn rooted_member_matchers_canonicalize_this_app_chains() {
    let source = r#"this.app.vault.read(file);"#;
    let program = parse_program(source);
    let rule = ApiRule::builder("test.canonical_app")
        .label("Canonical app chain")
        .category(ApiCategory::Vault)
        .severity(ApiSeverity::Info)
        .confidence(Confidence::High)
        .rooted_member_calls(["app.vault.read"])
        .build()
        .unwrap();
    let result = classify_api_usage(source, program.as_ref(), &[rule]);

    assert!(result.has_capability("test.canonical_app"));
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
