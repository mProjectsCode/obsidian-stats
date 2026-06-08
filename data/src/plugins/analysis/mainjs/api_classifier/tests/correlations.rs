use super::*;

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
            .severity(ApiSeverity::Info)
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
            .category(ApiCategory::Network)
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
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .when_all(["network.browser"])
            .build()
            .unwrap(),
        ApiRule::builder("network.browser")
            .label("Browser network")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .global_calls(["fetch"])
            .build()
            .unwrap(),
    ];

    let result = classify_api_usage(source, program.as_ref(), &rules);

    assert!(result.has_capability("correlation.network"));
}
