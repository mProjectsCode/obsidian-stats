use swc_ecma_ast::{CallExpr, Callee, Expr, MemberExpr, MemberProp, NewExpr, Program};
use swc_ecma_visit::{Visit, VisitWith};

pub(super) fn detect_webassembly_usage(source: &str, program: Option<&Program>) -> u32 {
    if let Some(program) = program {
        let mut visitor = WasmVisitor::default();
        program.visit_with(&mut visitor);
        return visitor.count;
    }

    let direct = source.matches("WebAssembly.").count();
    let typo = source.matches("WebAssembly.initiate(").count();
    (direct + typo) as u32
}

#[derive(Default)]
struct WasmVisitor {
    count: u32,
}

impl Visit for WasmVisitor {
    fn visit_call_expr(&mut self, expr: &CallExpr) {
        if let Callee::Expr(callee) = &expr.callee
            && let Expr::Member(member) = &**callee
            && is_webassembly_method(member)
        {
            self.count += 1;
        }

        expr.visit_children_with(self);
    }

    fn visit_new_expr(&mut self, expr: &NewExpr) {
        if let Expr::Member(member) = &*expr.callee
            && is_webassembly_constructor(member)
        {
            self.count += 1;
        }

        expr.visit_children_with(self);
    }
}

fn is_webassembly_method(member: &MemberExpr) -> bool {
    if !is_webassembly_object(&member.obj) {
        return false;
    }

    match &member.prop {
        MemberProp::Ident(ident) => {
            ident.sym == *"instantiate"
                || ident.sym == *"instantiateStreaming"
                || ident.sym == *"compile"
                || ident.sym == *"compileStreaming"
                || ident.sym == *"validate"
        }
        _ => false,
    }
}

fn is_webassembly_constructor(member: &MemberExpr) -> bool {
    if !is_webassembly_object(&member.obj) {
        return false;
    }

    matches!(
        &member.prop,
        MemberProp::Ident(ident)
            if ident.sym == *"Module"
                || ident.sym == *"Instance"
                || ident.sym == *"Memory"
                || ident.sym == *"Table"
                || ident.sym == *"Global"
    )
}

fn is_webassembly_object(expr: &Expr) -> bool {
    if let Expr::Ident(ident) = expr {
        ident.sym == *"WebAssembly"
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::detect_webassembly_usage;
    use crate::plugins::analysis::mainjs::parse_program;

    #[test]
    fn detects_common_webassembly_calls_and_constructors() {
        let source = r#"
            WebAssembly.compileStreaming(fetch("mod.wasm"));
            WebAssembly.validate(bytes);
            new WebAssembly.Instance(module, imports);
            new WebAssembly.Memory({ initial: 1 });
        "#;
        let program = parse_program(source);

        assert_eq!(detect_webassembly_usage(source, program.as_ref()), 4);
    }

    #[test]
    fn does_not_count_misspelled_initiate_api() {
        let source = "WebAssembly.initiate(bytes)";
        let program = parse_program(source);

        assert_eq!(detect_webassembly_usage(source, program.as_ref()), 0);
    }
}
