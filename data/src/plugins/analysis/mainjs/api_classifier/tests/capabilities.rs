use super::*;

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
    let result = classify_api_usage(program.as_ref(), obsidian_api_rules());

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
    let result = classify_api_usage(program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("network.obsidian"));
}

#[test]
fn minified_local_request_url_function_does_not_count_as_obsidian_network_api() {
    let source = r#"function requestUrl(r){return r}requestUrl("not-network");"#;
    let program = parse_program(source);
    let result = classify_api_usage(program.as_ref(), obsidian_api_rules());

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
    let result = classify_api_usage(program.as_ref(), obsidian_api_rules());

    assert!(!result.has_capability("network.browser"));
}

#[test]
fn minified_shadowed_fetch_does_not_count_as_browser_network_api() {
    let source = r#"function fetch(r){return r}fetch("not-network");"#;
    let program = parse_program(source);
    let result = classify_api_usage(program.as_ref(), obsidian_api_rules());

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
    let result = classify_api_usage(program.as_ref(), obsidian_api_rules());

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
    let result = classify_api_usage(program.as_ref(), obsidian_api_rules());

    assert!(result.has_capability("network.browser"));
}
