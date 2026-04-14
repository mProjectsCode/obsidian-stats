use swc_common::{FileName, SourceMap, sync::Lrc};
use swc_ecma_ast::{EsVersion as SwcEsVersion, Program};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax, lexer::Lexer};

use super::types::MainJsResult;

mod check_base64;
mod check_es;
mod check_minified;
mod check_sourcemap;
mod check_wasm;
mod check_worker;

pub(super) fn analyze_main_js(source: &str) -> MainJsResult {
    let mut result = MainJsResult::default();

    let program = parse_program(source);

    result.estimated_target_es_version = program.as_ref().and_then(check_es::detect_es_version);

    let (is_probably_minified, minification_score) =
        check_minified::detect_minified(source, program.as_ref());
    result.is_probably_minified = Some(is_probably_minified);
    result.minification_score = Some(minification_score);
    result.includes_sourcemap_comment = Some(check_sourcemap::detect_sourcemap_comment(source));

    let (large_base64_blob_count, largest_base64_blob_length) = check_base64::detect_base64(source);
    result.large_base64_blob_count = Some(large_base64_blob_count);
    result.largest_base64_blob_length = Some(largest_base64_blob_length);

    result.worker_usage_count = Some(check_worker::detect_worker_usage(source, program.as_ref()));
    result.webassembly_usage_count = Some(check_wasm::detect_webassembly_usage(
        source,
        program.as_ref(),
    ));

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
