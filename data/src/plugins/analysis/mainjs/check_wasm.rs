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
            && is_webassembly_module(member)
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
                || ident.sym == *"initiate"
        }
        _ => false,
    }
}

fn is_webassembly_module(member: &MemberExpr) -> bool {
    is_webassembly_object(&member.obj)
        && matches!(&member.prop, MemberProp::Ident(ident) if ident.sym == *"Module")
}

fn is_webassembly_object(expr: &Expr) -> bool {
    if let Expr::Ident(ident) = expr {
        ident.sym == *"WebAssembly"
    } else {
        false
    }
}
