use super::*;

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
