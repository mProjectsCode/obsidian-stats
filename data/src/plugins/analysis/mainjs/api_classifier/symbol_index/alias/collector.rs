use std::collections::{BTreeMap, BTreeSet};

use swc_common::Span;
use swc_ecma_ast::{
    ArrowExpr, AssignExpr, AssignTarget, BlockStmt, CallExpr, Callee, CatchClause, ClassDecl, Expr,
    FnDecl, Function, ImportDecl, ImportSpecifier, Pat, SimpleAssignTarget, VarDecl, VarDeclKind,
};
use swc_ecma_visit::{Visit, VisitWith};

use super::super::ast::{
    binding_ident_name, collect_pat_bindings, member_chain, member_prop_name, module_export_name,
    require_module_name,
};
use super::collector_helpers::{
    collect_assignment_aliases, collect_require_aliases, collect_value_aliases,
};
use super::{AliasAssignment, AliasScope, BindingProvenance, PropertyAliasAssignment, ScopeKind};

pub(super) struct AliasCollector {
    pub(super) scopes: Vec<AliasScope>,
    stack: Vec<usize>,
    pub(super) assignments: Vec<AliasAssignment>,
    latest_assignments: BTreeMap<usize, BTreeMap<String, BindingProvenance>>,
    pub(super) property_assignments: Vec<PropertyAliasAssignment>,
    functions: BTreeMap<String, (usize, Vec<String>)>,
    calls: Vec<(String, Vec<Option<String>>)>,
}

impl AliasCollector {
    pub(super) fn new(program_span: Span) -> Self {
        Self {
            scopes: vec![AliasScope {
                span: program_span,
                depth: 0,
                kind: ScopeKind::Program,
                parent: None,
                bindings: BTreeMap::new(),
            }],
            stack: vec![0],
            assignments: Vec::new(),
            latest_assignments: BTreeMap::new(),
            property_assignments: Vec::new(),
            functions: BTreeMap::new(),
            calls: Vec::new(),
        }
    }

    fn current_scope(&self) -> usize {
        *self.stack.last().expect("program scope is always present")
    }

    fn binding_scope(&self, kind: VarDeclKind) -> usize {
        if kind != VarDeclKind::Var {
            return self.current_scope();
        }
        self.stack
            .iter()
            .rev()
            .copied()
            .find(|index| {
                matches!(
                    self.scopes[*index].kind,
                    ScopeKind::Program | ScopeKind::Function
                )
            })
            .expect("program scope is always present")
    }

    pub(super) fn insert(
        &mut self,
        scope: usize,
        name: impl Into<String>,
        provenance: BindingProvenance,
    ) {
        self.scopes[scope].bindings.insert(name.into(), provenance);
    }

    fn insert_local(&mut self, scope: usize, name: impl Into<String>) {
        self.insert(scope, name, BindingProvenance::Local);
    }

    pub(super) fn record_assignment(
        &mut self,
        span: Span,
        scope: usize,
        name: String,
        provenance: BindingProvenance,
    ) {
        self.latest_assignments
            .entry(scope)
            .or_default()
            .insert(name.clone(), provenance.clone());
        self.assignments.push(AliasAssignment {
            span,
            scope,
            name,
            provenance,
        });
    }

    fn push_scope(&mut self, span: Span, kind: ScopeKind) {
        let index = self.scopes.len();
        let parent = self.current_scope();
        self.scopes.push(AliasScope {
            span,
            depth: self.stack.len(),
            kind,
            parent: Some(parent),
            bindings: BTreeMap::new(),
        });
        self.stack.push(index);
    }

    fn pop_scope(&mut self) {
        self.stack.pop();
    }

    fn insert_pat_locals(&mut self, scope: usize, pat: &Pat) {
        let mut bindings = BTreeSet::new();
        collect_pat_bindings(pat, &mut bindings);
        for binding in bindings {
            self.insert_local(scope, binding);
        }
    }

    fn visible_binding(&self, name: &str) -> Option<&BindingProvenance> {
        for scope in self.stack.iter().rev().copied() {
            if let Some(assignment) = self
                .latest_assignments
                .get(&scope)
                .and_then(|assignments| assignments.get(name))
            {
                return Some(assignment);
            }
            if let Some(binding) = self.scopes[scope].bindings.get(name) {
                return Some(binding);
            }
        }
        None
    }

    fn rooted_expr_name(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::This(_) => Some("this".to_string()),
            Expr::Ident(ident) => match self.visible_binding(ident.sym.as_ref()) {
                Some(BindingProvenance::ValueAlias { target }) => Some(target.clone()),
                Some(_) => None,
                None => Some(ident.sym.to_string()),
            },
            Expr::Member(member) => {
                let object = self.rooted_expr_name(&member.obj)?;
                let property = member_prop_name(&member.prop)?;
                Some(format!("{object}.{property}"))
            }
            Expr::Paren(paren) => self.rooted_expr_name(&paren.expr),
            _ => None,
        }
    }

    pub(super) fn parameter_aliases(&self) -> BTreeMap<(usize, String), BindingProvenance> {
        let mut aliases = BTreeMap::<(usize, String), Option<String>>::new();
        for (callee, arguments) in &self.calls {
            let Some((scope, parameters)) = self.functions.get(callee) else {
                continue;
            };
            for (parameter, target) in parameters.iter().zip(arguments) {
                let entry = aliases
                    .entry((*scope, parameter.clone()))
                    .or_insert_with(|| target.clone());
                if entry != target {
                    *entry = None;
                }
            }
        }
        aliases
            .into_iter()
            .filter_map(|(key, target)| {
                target.map(|target| (key, BindingProvenance::ValueAlias { target }))
            })
            .collect()
    }
}

impl Visit for AliasCollector {
    fn visit_import_decl(&mut self, import: &ImportDecl) {
        let scope = self.current_scope();
        let module = import.src.value.to_string_lossy().to_string();
        for specifier in &import.specifiers {
            match specifier {
                ImportSpecifier::Named(named) => {
                    let local = named.local.sym.to_string();
                    let export = named
                        .imported
                        .as_ref()
                        .map(module_export_name)
                        .unwrap_or_else(|| local.clone());
                    self.insert(
                        scope,
                        local,
                        BindingProvenance::ModuleExport {
                            module: module.clone(),
                            export,
                        },
                    );
                }
                ImportSpecifier::Namespace(namespace) => self.insert(
                    scope,
                    namespace.local.sym.to_string(),
                    BindingProvenance::ModuleNamespace {
                        module: module.clone(),
                    },
                ),
                ImportSpecifier::Default(default) => {
                    self.insert_local(scope, default.local.sym.to_string());
                }
            }
        }
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        let scope = self.binding_scope(var_decl.kind);
        for declarator in &var_decl.decls {
            let value_alias = declarator
                .init
                .as_deref()
                .and_then(|init| self.rooted_expr_name(init));
            self.insert_pat_locals(scope, &declarator.name);
            if let Some(init) = declarator.init.as_deref()
                && let Some(module) = require_module_name(init)
            {
                collect_require_aliases(&declarator.name, module, scope, self);
            } else if let Some(target) = value_alias {
                collect_value_aliases(&declarator.name, &target, scope, self);
            }
        }
        var_decl.visit_children_with(self);
    }

    fn visit_assign_expr(&mut self, assignment: &AssignExpr) {
        let provenance = self
            .rooted_expr_name(&assignment.right)
            .map(|target| BindingProvenance::ValueAlias { target })
            .unwrap_or(BindingProvenance::Local);
        match &assignment.left {
            AssignTarget::Simple(SimpleAssignTarget::Ident(ident)) => {
                if let Some((scope, _)) = self.stack.iter().rev().find_map(|scope| {
                    self.scopes[*scope]
                        .bindings
                        .contains_key(ident.id.sym.as_ref())
                        .then_some((*scope, ()))
                }) {
                    self.record_assignment(
                        assignment.span,
                        scope,
                        ident.id.sym.to_string(),
                        provenance,
                    );
                }
            }
            AssignTarget::Simple(SimpleAssignTarget::Member(member)) => {
                if let Some(property) = member_chain(member) {
                    self.property_assignments.push(PropertyAliasAssignment {
                        span: assignment.span,
                        scope: self.current_scope(),
                        property,
                        target: self.rooted_expr_name(&assignment.right),
                    });
                }
            }
            AssignTarget::Pat(pattern) => {
                let pattern: Pat = pattern.clone().into();
                if let Some(target) = self.rooted_expr_name(&assignment.right) {
                    collect_assignment_aliases(
                        &pattern,
                        &target,
                        assignment.span,
                        self.current_scope(),
                        self,
                    );
                }
            }
            _ => {}
        }
        assignment.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, call: &CallExpr) {
        if let Callee::Expr(callee) = &call.callee
            && let Expr::Ident(callee) = &**callee
        {
            self.calls.push((
                callee.sym.to_string(),
                call.args
                    .iter()
                    .map(|argument| self.rooted_expr_name(&argument.expr))
                    .collect(),
            ));
        }
        call.visit_children_with(self);
    }

    fn visit_fn_decl(&mut self, fn_decl: &FnDecl) {
        let parent = self.current_scope();
        self.insert_local(parent, fn_decl.ident.sym.to_string());
        self.push_scope(fn_decl.function.span, ScopeKind::Function);
        let scope = self.current_scope();
        let parameters = fn_decl
            .function
            .params
            .iter()
            .filter_map(|parameter| binding_ident_name(&parameter.pat))
            .collect::<Vec<_>>();
        for parameter in &fn_decl.function.params {
            self.insert_pat_locals(scope, &parameter.pat);
        }
        self.functions
            .insert(fn_decl.ident.sym.to_string(), (scope, parameters));
        fn_decl.function.decorators.visit_with(self);
        fn_decl.function.body.visit_with(self);
        self.pop_scope();
    }

    fn visit_class_decl(&mut self, class_decl: &ClassDecl) {
        let scope = self.current_scope();
        self.insert_local(scope, class_decl.ident.sym.to_string());
        class_decl.class.visit_children_with(self);
    }

    fn visit_function(&mut self, function: &Function) {
        self.push_scope(function.span, ScopeKind::Function);
        let scope = self.current_scope();
        for param in &function.params {
            self.insert_pat_locals(scope, &param.pat);
        }
        function.decorators.visit_with(self);
        function.body.visit_with(self);
        self.pop_scope();
    }

    fn visit_arrow_expr(&mut self, arrow: &ArrowExpr) {
        self.push_scope(arrow.span, ScopeKind::Function);
        let scope = self.current_scope();
        for param in &arrow.params {
            self.insert_pat_locals(scope, param);
        }
        arrow.body.visit_with(self);
        self.pop_scope();
    }

    fn visit_block_stmt(&mut self, block: &BlockStmt) {
        self.push_scope(block.span, ScopeKind::Block);
        block.stmts.visit_with(self);
        self.pop_scope();
    }

    fn visit_catch_clause(&mut self, catch: &CatchClause) {
        self.push_scope(catch.span, ScopeKind::Block);
        let scope = self.current_scope();
        if let Some(param) = &catch.param {
            self.insert_pat_locals(scope, param);
        }
        catch.body.stmts.visit_with(self);
        self.pop_scope();
    }
}
