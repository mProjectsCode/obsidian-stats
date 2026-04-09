use std::fs;

use swc_common::{FileName, SourceMap, sync::Lrc};
use swc_ecma_ast::{
    ArrayPat, ArrowExpr, AssignExpr, AssignOp, AwaitExpr, BinExpr, BinaryOp, CatchClause, Class,
    ClassMember, EsVersion as SwcEsVersion, ExportDecl, ForOfStmt, Function, ImportDecl, ObjectLit,
    ObjectPat, OptChainExpr, PrivateName, Program, StaticBlock, VarDecl, VarDeclKind,
};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax, lexer::Lexer};
use swc_ecma_visit::{Visit, VisitWith};

use crate::plugins::release_acquisition::release_main_js_cache_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DetectedEsVersion {
    Es5,
    Es2015,
    Es2016,
    Es2017,
    Es2018,
    Es2019,
    Es2020,
    Es2021,
    Es2022,
}

impl DetectedEsVersion {
    fn as_label(self) -> &'static str {
        match self {
            Self::Es5 => "ES5",
            Self::Es2015 => "ES6",
            Self::Es2016 => "ES2016",
            Self::Es2017 => "ES2017",
            Self::Es2018 => "ES2018",
            Self::Es2019 => "ES2019",
            Self::Es2020 => "ES2020",
            Self::Es2021 => "ES2021",
            Self::Es2022 => "ES2022",
        }
    }
}

#[derive(Debug)]
struct EsFeatureDetector {
    detected: DetectedEsVersion,
    function_depth: usize,
}

impl Default for EsFeatureDetector {
    fn default() -> Self {
        Self {
            detected: DetectedEsVersion::Es5,
            function_depth: 0,
        }
    }
}

impl EsFeatureDetector {
    fn bump(&mut self, version: DetectedEsVersion) {
        if version > self.detected {
            self.detected = version;
        }
    }
}

impl Visit for EsFeatureDetector {
    fn visit_program(&mut self, node: &Program) {
        node.visit_children_with(self);
    }

    fn visit_arrow_expr(&mut self, node: &ArrowExpr) {
        self.bump(DetectedEsVersion::Es2015);
        if node.is_async {
            self.bump(DetectedEsVersion::Es2017);
        }

        self.function_depth += 1;
        node.visit_children_with(self);
        self.function_depth -= 1;
    }

    fn visit_function(&mut self, node: &Function) {
        if node.is_generator {
            self.bump(DetectedEsVersion::Es2015);
        }
        if node.is_async {
            self.bump(DetectedEsVersion::Es2017);
        }

        self.function_depth += 1;
        node.visit_children_with(self);
        self.function_depth -= 1;
    }

    fn visit_await_expr(&mut self, node: &AwaitExpr) {
        if self.function_depth == 0 {
            self.bump(DetectedEsVersion::Es2022);
        } else {
            self.bump(DetectedEsVersion::Es2017);
        }

        node.visit_children_with(self);
    }

    fn visit_var_decl(&mut self, node: &VarDecl) {
        if node.kind != VarDeclKind::Var {
            self.bump(DetectedEsVersion::Es2015);
        }

        node.visit_children_with(self);
    }

    fn visit_array_pat(&mut self, node: &ArrayPat) {
        self.bump(DetectedEsVersion::Es2015);
        node.visit_children_with(self);
    }

    fn visit_object_pat(&mut self, node: &ObjectPat) {
        self.bump(DetectedEsVersion::Es2015);
        node.visit_children_with(self);
    }

    fn visit_class(&mut self, node: &Class) {
        self.bump(DetectedEsVersion::Es2015);
        node.visit_children_with(self);
    }

    fn visit_class_member(&mut self, node: &ClassMember) {
        match node {
            ClassMember::ClassProp(_) => self.bump(DetectedEsVersion::Es2022),
            ClassMember::PrivateMethod(_) => self.bump(DetectedEsVersion::Es2022),
            ClassMember::PrivateProp(_) => self.bump(DetectedEsVersion::Es2022),
            ClassMember::StaticBlock(_) => self.bump(DetectedEsVersion::Es2022),
            ClassMember::AutoAccessor(_) => self.bump(DetectedEsVersion::Es2022),
            _ => {}
        }

        node.visit_children_with(self);
    }

    fn visit_static_block(&mut self, node: &StaticBlock) {
        self.bump(DetectedEsVersion::Es2022);
        node.visit_children_with(self);
    }

    fn visit_private_name(&mut self, node: &PrivateName) {
        self.bump(DetectedEsVersion::Es2022);
        node.visit_children_with(self);
    }

    fn visit_for_of_stmt(&mut self, node: &ForOfStmt) {
        self.bump(DetectedEsVersion::Es2015);
        if node.is_await {
            self.bump(DetectedEsVersion::Es2018);
        }

        node.visit_children_with(self);
    }

    fn visit_object_lit(&mut self, node: &ObjectLit) {
        if node
            .props
            .iter()
            .any(|prop| matches!(prop, swc_ecma_ast::PropOrSpread::Spread(_)))
        {
            self.bump(DetectedEsVersion::Es2018);
        }

        node.visit_children_with(self);
    }

    fn visit_import_decl(&mut self, node: &ImportDecl) {
        self.bump(DetectedEsVersion::Es2015);
        node.visit_children_with(self);
    }

    fn visit_export_decl(&mut self, node: &ExportDecl) {
        self.bump(DetectedEsVersion::Es2015);
        node.visit_children_with(self);
    }

    fn visit_bin_expr(&mut self, node: &BinExpr) {
        match node.op {
            BinaryOp::Exp => self.bump(DetectedEsVersion::Es2016),
            BinaryOp::NullishCoalescing => self.bump(DetectedEsVersion::Es2020),
            _ => {}
        }

        node.visit_children_with(self);
    }

    fn visit_assign_expr(&mut self, node: &AssignExpr) {
        match node.op {
            AssignOp::ExpAssign => self.bump(DetectedEsVersion::Es2016),
            AssignOp::AndAssign | AssignOp::OrAssign | AssignOp::NullishAssign => {
                self.bump(DetectedEsVersion::Es2021)
            }
            _ => {}
        }

        node.visit_children_with(self);
    }

    fn visit_opt_chain_expr(&mut self, node: &OptChainExpr) {
        self.bump(DetectedEsVersion::Es2020);
        node.visit_children_with(self);
    }

    fn visit_catch_clause(&mut self, node: &CatchClause) {
        if node.param.is_none() {
            self.bump(DetectedEsVersion::Es2019);
        }

        node.visit_children_with(self);
    }
}

fn parse_program(source: &str) -> Option<Program> {
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

pub(super) fn infer_es_from_main_js(plugin_id: &str, release_tag: &str) -> Option<String> {
    let path = release_main_js_cache_path(plugin_id, release_tag);
    let bytes = fs::read(path).ok()?;
    let source = std::str::from_utf8(&bytes).ok()?;

    infer_es_from_js_source(source)
}

pub(super) fn infer_es_from_js_source(source: &str) -> Option<String> {
    let program = parse_program(source)?;

    let mut detector = EsFeatureDetector::default();
    program.visit_with(&mut detector);

    Some(detector.detected.as_label().to_string())
}

#[cfg(test)]
mod tests {
    use super::infer_es_from_js_source;

    #[test]
    fn defaults_to_es5_when_no_modern_syntax() {
        let src = r#"var total = 1 + 2; function f() { return total; }"#;

        assert_eq!(infer_es_from_js_source(src).as_deref(), Some("ES5"));
    }

    #[test]
    fn detects_es2015_features() {
        let src = r#"const map = values => values.map(v => v * 2);"#;

        assert_eq!(infer_es_from_js_source(src).as_deref(), Some("ES6"));
    }

    #[test]
    fn detects_es2016_features() {
        let src = r#"let value = 2 ** 4; value **= 2;"#;

        assert_eq!(infer_es_from_js_source(src).as_deref(), Some("ES2016"));
    }

    #[test]
    fn detects_es2017_features() {
        let src = r#"async function load() { await fetch('x'); }"#;

        assert_eq!(infer_es_from_js_source(src).as_deref(), Some("ES2017"));
    }

    #[test]
    fn detects_es2018_features() {
        let src = r#"const merged = { ...a, b: 1 };"#;

        assert_eq!(infer_es_from_js_source(src).as_deref(), Some("ES2018"));
    }

    #[test]
    fn detects_es2019_features() {
        let src = r#"try { fn(); } catch { noop(); }"#;

        assert_eq!(infer_es_from_js_source(src).as_deref(), Some("ES2019"));
    }

    #[test]
    fn detects_es2020_features() {
        let src = r#"const value = user?.profile ?? fallback;"#;

        assert_eq!(infer_es_from_js_source(src).as_deref(), Some("ES2020"));
    }

    #[test]
    fn detects_es2021_features() {
        let src = r#"config.enabled &&= true; count ||= 1; result ??= 0;"#;

        assert_eq!(infer_es_from_js_source(src).as_deref(), Some("ES2021"));
    }

    #[test]
    fn detects_es2022_features() {
        let src = r#"class Example { #value = 1; }"#;

        assert_eq!(infer_es_from_js_source(src).as_deref(), Some("ES2022"));
    }

    #[test]
    fn ignores_runtime_api_mentions_inside_strings_and_comments() {
        let src = r#"
            // Promise.allSettled and Object.fromEntries are runtime APIs, not syntax.
            const text = "Promise.allSettled Object.fromEntries";
            var keep = 1;
        "#;

        assert_eq!(infer_es_from_js_source(src).as_deref(), Some("ES6"));
    }

    #[test]
    fn returns_none_for_unparseable_source() {
        let src = "function (";

        assert_eq!(infer_es_from_js_source(src), None);
    }
}
