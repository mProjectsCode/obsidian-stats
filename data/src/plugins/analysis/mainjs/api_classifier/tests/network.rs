use super::*;

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
