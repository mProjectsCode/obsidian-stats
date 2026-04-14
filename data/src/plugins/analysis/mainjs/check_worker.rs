use swc_ecma_ast::{CallExpr, Callee, Expr, MemberExpr, MemberProp, NewExpr, Program};
use swc_ecma_visit::{Visit, VisitWith};

pub(super) fn detect_worker_usage(source: &str, program: Option<&Program>) -> u32 {
    if let Some(program) = program {
        let mut visitor = WorkerVisitor::default();
        program.visit_with(&mut visitor);
        return visitor.count;
    }

    let worker_calls = source.matches("Worker(").count();
    let service_worker = source.matches("serviceWorker").count();

    (worker_calls + service_worker) as u32
}

#[derive(Default)]
struct WorkerVisitor {
    count: u32,
}

impl Visit for WorkerVisitor {
    fn visit_new_expr(&mut self, expr: &NewExpr) {
        if let Expr::Ident(ident) = &*expr.callee
            && (ident.sym == *"Worker" || ident.sym == *"SharedWorker")
        {
            self.count += 1;
        }

        expr.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) {
        match &expr.callee {
            Callee::Expr(callee) => {
                if let Expr::Ident(ident) = &**callee
                    && (ident.sym == *"Worker" || ident.sym == *"SharedWorker")
                {
                    self.count += 1;
                }
            }
            Callee::Super(_) | Callee::Import(_) => {}
        }

        expr.visit_children_with(self);
    }

    fn visit_member_expr(&mut self, member: &MemberExpr) {
        if is_service_worker(member) {
            self.count += 1;
        }

        member.visit_children_with(self);
    }
}

fn is_service_worker(member: &MemberExpr) -> bool {
    let obj_is_navigator = if let Expr::Ident(ident) = &*member.obj {
        ident.sym == *"navigator"
    } else {
        false
    };

    if !obj_is_navigator {
        return false;
    }

    match &member.prop {
        MemberProp::Ident(ident) => ident.sym == *"serviceWorker",
        _ => false,
    }
}
