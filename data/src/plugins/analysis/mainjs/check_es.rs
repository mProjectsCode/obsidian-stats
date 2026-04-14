use swc_ecma_ast::{
    ArrayPat, ArrowExpr, AssignExpr, AssignOp, AwaitExpr, BinExpr, BinaryOp, CatchClause, Class,
    ClassMember, ExportDecl, ForOfStmt, Function, ImportDecl, ObjectLit, ObjectPat, OptChainExpr,
    PrivateName, Program, StaticBlock, VarDecl, VarDeclKind,
};
use swc_ecma_visit::{Visit, VisitWith};

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

pub(super) fn detect_es_version(program: &Program) -> Option<String> {
    let mut detector = EsFeatureDetector::default();
    program.visit_with(&mut detector);

    Some(detector.detected.as_label().to_string())
}
