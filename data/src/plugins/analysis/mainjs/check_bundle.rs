use swc_ecma_ast::Program;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct BundleShape {
    pub(super) parse_succeeded: bool,
    pub(super) tolerant_parse_required: bool,
    pub(super) dynamic_import_count: u32,
    pub(super) bundler_fingerprints: Vec<String>,
    pub(super) module_system_fingerprints: Vec<String>,
    pub(super) size_bucket: String,
    pub(super) line_count_bucket: String,
    pub(super) uses_optional_chaining: bool,
    pub(super) uses_nullish_coalescing: bool,
    pub(super) uses_private_fields: bool,
    pub(super) uses_top_level_await: bool,
}

pub(super) fn detect_bundle_shape(source: &str, program: Option<&Program>) -> BundleShape {
    BundleShape {
        parse_succeeded: program.is_some(),
        tolerant_parse_required: false,
        dynamic_import_count: count_marker(source, "import("),
        bundler_fingerprints: detect_bundler_fingerprints(source),
        module_system_fingerprints: detect_module_system_fingerprints(source),
        size_bucket: size_bucket(source.len()).to_string(),
        line_count_bucket: line_count_bucket(source.lines().count()).to_string(),
        uses_optional_chaining: source.contains("?."),
        uses_nullish_coalescing: source.contains("??"),
        uses_private_fields: source.contains("this.#") || source.contains(" #"),
        uses_top_level_await: program
            .and_then(crate::plugins::analysis::mainjs::check_es::detect_es_version)
            .as_deref()
            == Some("ES2022")
            && source.contains("await"),
    }
}

fn detect_bundler_fingerprints(source: &str) -> Vec<String> {
    let mut fingerprints = Vec::new();
    push_if(
        source,
        &mut fingerprints,
        "esbuild",
        ["__toESM", "__commonJS"],
    );
    push_if(source, &mut fingerprints, "rollup", ["rollup", "__defProp"]);
    push_if(
        source,
        &mut fingerprints,
        "webpack",
        ["__webpack_require__", "webpackJsonp"],
    );
    push_if(
        source,
        &mut fingerprints,
        "vite",
        ["import.meta.env", "__vite"],
    );
    push_if(
        source,
        &mut fingerprints,
        "parcel",
        ["parcelRequire", "__parcel"],
    );
    push_if(
        source,
        &mut fingerprints,
        "browserify",
        ["function r(e,n,t)", "browserify"],
    );
    push_if(
        source,
        &mut fingerprints,
        "swc",
        ["@swc/helpers", "_class_call_check"],
    );
    push_if(
        source,
        &mut fingerprints,
        "terser",
        ["sourceMappingURL", "terser"],
    );
    fingerprints
}

fn detect_module_system_fingerprints(source: &str) -> Vec<String> {
    let mut fingerprints = Vec::new();
    push_if(
        source,
        &mut fingerprints,
        "commonjs",
        ["require(", "module.exports", "exports."],
    );
    push_if(source, &mut fingerprints, "amd", ["define.amd", "define("]);
    push_if(
        source,
        &mut fingerprints,
        "umd",
        ["typeof exports", "typeof define"],
    );
    push_if(source, &mut fingerprints, "esm", ["import ", "export "]);
    fingerprints
}

fn push_if<const N: usize>(
    source: &str,
    values: &mut Vec<String>,
    label: &str,
    markers: [&str; N],
) {
    if markers.iter().any(|marker| source.contains(marker)) {
        values.push(label.to_string());
    }
}

fn size_bucket(size: usize) -> &'static str {
    match size {
        0..=99_999 => "under_100kb",
        100_000..=499_999 => "100kb_to_500kb",
        500_000..=999_999 => "500kb_to_1mb",
        1_000_000..=4_999_999 => "1mb_to_5mb",
        _ => "over_5mb",
    }
}

fn line_count_bucket(lines: usize) -> &'static str {
    match lines {
        0..=999 => "under_1k",
        1_000..=4_999 => "1k_to_5k",
        5_000..=19_999 => "5k_to_20k",
        _ => "over_20k",
    }
}

fn count_marker(source: &str, marker: &str) -> u32 {
    source
        .matches(marker)
        .count()
        .try_into()
        .unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::detect_bundle_shape;
    use crate::plugins::analysis::mainjs::parse_program;

    #[test]
    fn detects_bundle_shape_markers() {
        let source = r#"
            const plugin = require("obsidian");
            const m = await import("./worker.js");
            const x = value?.name ?? "fallback";
            class Example { #secret = 1; }
            var o = __toESM(require("obsidian"));
            //# sourceMappingURL=data:application/json;base64,AAAA
        "#;
        let program = parse_program(source);
        let shape = detect_bundle_shape(source, program.as_ref());

        assert!(shape.parse_succeeded);
        assert_eq!(shape.dynamic_import_count, 1);
        assert!(shape.bundler_fingerprints.contains(&"esbuild".to_string()));
        assert!(
            shape
                .module_system_fingerprints
                .contains(&"commonjs".to_string())
        );
        assert!(shape.uses_optional_chaining);
        assert!(shape.uses_nullish_coalescing);
        assert!(shape.uses_private_fields);
    }
}
