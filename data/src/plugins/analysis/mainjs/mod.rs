use swc_common::{FileName, SourceMap, sync::Lrc};
use swc_ecma_ast::{EsVersion as SwcEsVersion, Program};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax, lexer::Lexer};

use super::types::MainJsResult;

pub(super) mod api_classifier;
mod check_base64;
mod check_bundle;
mod check_es;
mod check_minified;
mod check_sourcemap;
mod check_strings;
mod check_wasm;
mod check_worker;

pub(super) fn analyze_main_js(source: &str) -> MainJsResult {
    let mut result = MainJsResult::default();

    let program = parse_program(source);
    let bundle_shape = check_bundle::detect_bundle_shape(source, program.as_ref());
    let string_signals = check_strings::detect_string_signals(source);

    result.parse_succeeded = Some(bundle_shape.parse_succeeded);
    result.tolerant_parse_required = Some(bundle_shape.tolerant_parse_required);
    result.estimated_target_es_version = program.as_ref().and_then(check_es::detect_es_version);
    result.dynamic_import_usage_count = Some(bundle_shape.dynamic_import_count);
    result.bundler_fingerprints = bundle_shape.bundler_fingerprints;
    result.module_system_fingerprints = bundle_shape.module_system_fingerprints;
    result.size_bucket = Some(bundle_shape.size_bucket);
    result.line_count_bucket = Some(bundle_shape.line_count_bucket);
    result.uses_optional_chaining = Some(bundle_shape.uses_optional_chaining);
    result.uses_nullish_coalescing = Some(bundle_shape.uses_nullish_coalescing);
    result.uses_private_fields = Some(bundle_shape.uses_private_fields);
    result.uses_top_level_await = Some(bundle_shape.uses_top_level_await);
    result.known_api_host_counts = string_signals.known_api_host_counts;
    result.embedded_dependency_name_counts = string_signals.dependency_name_counts;
    result.license_banner_count = Some(string_signals.license_banner_count);
    result.credential_literal_count = Some(string_signals.credential_literal_count);

    let (is_probably_minified, minification_score) =
        check_minified::detect_minified(source, program.as_ref());
    result.is_probably_minified = Some(is_probably_minified);
    result.minification_score = Some(minification_score);
    let api_rules = api_classifier::obsidian_api_rules();
    if !api_rules.is_empty() {
        debug_assert!(api_classifier::validate_catalog(api_rules).is_ok());
        result.api_usage =
            api_classifier::classify_api_usage_with_source(source, program.as_ref(), api_rules);
    }

    result
}

pub(super) fn parse_program(source: &str) -> Option<Program> {
    let cm = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(
        FileName::Custom("main.js".into()).into(),
        source.to_string(),
    );

    let mut parser = Parser::new_from(Lexer::new(
        Syntax::Es(EsSyntax {
            jsx: true,
            fn_bind: true,
            decorators: true,
            decorators_before_export: true,
            export_default_from: true,
            import_attributes: true,
            allow_super_outside_method: true,
            allow_return_outside_function: true,
            auto_accessors: true,
            explicit_resource_management: true,
        }),
        SwcEsVersion::EsNext,
        StringInput::from(&*fm),
        None,
    ));

    parser.parse_program().ok()
}

#[cfg(test)]
mod tests {
    use super::analyze_main_js;

    #[test]
    fn detects_network_api_usage() {
        let result = analyze_main_js(
            r#"
            async function load() {
                await fetch("https://example.com");
            }
            "#,
        );

        assert!(result.api_usage.has_capability("network.browser"));
        assert!(result.api_usage.has_disclosure("disclosure.network_access"));
    }

    #[test]
    fn emits_bundle_signals_as_api_findings() {
        let base64 = "A".repeat(1024);
        let source = format!(
            r#"
            new Worker("worker.js");
            WebAssembly.compile(bytes);
            const blob = "{base64}";
            //# sourceMappingURL=main.js.map
            "#
        );
        let result = analyze_main_js(&source);

        for capability in [
            "bundle.source_map_comment",
            "bundle.embedded_base64_blob",
            "browser.worker",
            "browser.webassembly",
        ] {
            assert!(result.api_usage.has_capability(capability));
        }
        for disclosure in [
            "disclosure.source_map_comment",
            "disclosure.embedded_base64_blob",
            "disclosure.worker_usage",
            "disclosure.webassembly_usage",
        ] {
            assert!(result.api_usage.has_disclosure(disclosure));
        }
    }
}
