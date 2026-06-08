use std::collections::BTreeMap;

use swc_common::{Span, Spanned};
use swc_ecma_ast::{Expr, Ident, MemberExpr, OptChainBase, Program};
use swc_ecma_visit::VisitWith;

use super::ast::{SymbolCallProvenance, SymbolMemberProvenance, member_chain, member_root_ident};
use collector::AliasCollector;
use collector_helpers::{contains, member_prefix_ends};

mod collector;
mod collector_helpers;

#[derive(Debug, Default, Clone)]
pub(in crate::plugins::analysis::mainjs::api_classifier) struct AliasInfo {
    scopes: Vec<AliasScope>,
    scopes_by_start: Vec<usize>,
    assignments: BTreeMap<usize, BTreeMap<String, Vec<AliasAssignment>>>,
    property_assignments: BTreeMap<String, Vec<PropertyAliasAssignment>>,
    parameter_aliases: BTreeMap<(usize, String), BindingProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::plugins::analysis::mainjs::api_classifier) struct BindingKey {
    name: String,
    scope: Option<usize>,
}

impl std::fmt::Display for BindingKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(formatter)
    }
}

#[derive(Debug, Clone)]
struct AliasScope {
    span: Span,
    depth: usize,
    kind: ScopeKind,
    parent: Option<usize>,
    bindings: BTreeMap<String, BindingProvenance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeKind {
    Program,
    Function,
    Block,
}

#[derive(Debug, Clone)]
enum BindingProvenance {
    Local,
    ValueAlias { target: String },
    ModuleExport { module: String, export: String },
    ModuleNamespace { module: String },
}

#[derive(Debug, Clone)]
struct AliasAssignment {
    span: Span,
    scope: usize,
    name: String,
    provenance: BindingProvenance,
}

#[derive(Debug, Clone)]
struct PropertyAliasAssignment {
    span: Span,
    scope: usize,
    property: String,
    target: Option<String>,
}

impl AliasInfo {
    pub(in crate::plugins::analysis::mainjs::api_classifier) fn collect(program: &Program) -> Self {
        let mut collector = AliasCollector::new(program.span());
        program.visit_children_with(&mut collector);
        let parameter_aliases = collector.parameter_aliases();
        let mut scopes_by_start = (0..collector.scopes.len()).collect::<Vec<_>>();
        scopes_by_start.sort_by_key(|index| {
            let scope = &collector.scopes[*index];
            (scope.span.lo, scope.depth)
        });
        let mut assignments = BTreeMap::<usize, BTreeMap<String, Vec<AliasAssignment>>>::new();
        for assignment in collector.assignments {
            assignments
                .entry(assignment.scope)
                .or_default()
                .entry(assignment.name.clone())
                .or_default()
                .push(assignment);
        }
        for scope_assignments in assignments.values_mut() {
            for binding_assignments in scope_assignments.values_mut() {
                binding_assignments.sort_by_key(|assignment| assignment.span.lo);
            }
        }
        let mut property_assignments = BTreeMap::<String, Vec<PropertyAliasAssignment>>::new();
        for assignment in collector.property_assignments {
            property_assignments
                .entry(assignment.property.clone())
                .or_default()
                .push(assignment);
        }
        for assignments in property_assignments.values_mut() {
            assignments.sort_by_key(|assignment| assignment.span.lo);
        }
        Self {
            scopes: collector.scopes,
            scopes_by_start,
            assignments,
            property_assignments,
            parameter_aliases,
        }
    }

    pub(in crate::plugins::analysis::mainjs::api_classifier) fn call_provenance(
        &self,
        name: &str,
        span: Span,
    ) -> SymbolCallProvenance {
        match self.binding_at(name, span) {
            Some(BindingProvenance::ModuleExport { module, export }) => {
                SymbolCallProvenance::ModuleExport {
                    module: module.clone(),
                    export: export.clone(),
                }
            }
            Some(
                BindingProvenance::Local
                | BindingProvenance::ValueAlias { .. }
                | BindingProvenance::ModuleNamespace { .. },
            ) => SymbolCallProvenance::Local,
            None => SymbolCallProvenance::Global,
        }
    }

    pub(in crate::plugins::analysis::mainjs::api_classifier) fn member_call_provenance(
        &self,
        member: &MemberExpr,
    ) -> Option<SymbolMemberProvenance> {
        let chain = member_chain(member)?;
        self.member_call_provenance_from_raw(member, &chain)
    }

    pub(super) fn member_call_provenance_from_raw(
        &self,
        member: &MemberExpr,
        chain: &str,
    ) -> Option<SymbolMemberProvenance> {
        let root = member_root_ident(member)?;
        let member = chain.strip_prefix(root.sym.as_ref())?.strip_prefix('.')?;
        match self.binding_at(root.sym.as_ref(), root.span) {
            Some(BindingProvenance::ModuleNamespace { module }) => {
                Some(SymbolMemberProvenance::ModuleNamespace {
                    module: module.clone(),
                    member: member.to_string(),
                })
            }
            _ => None,
        }
    }

    fn binding_at(&self, name: &str, span: Span) -> Option<&BindingProvenance> {
        let (scope, declaration) = self.binding_with_scope_at(name, span)?;
        self.assignments
            .get(&scope)
            .and_then(|assignments| assignments.get(name))
            .and_then(|assignments| {
                assignments
                    .partition_point(|assignment| assignment.span.lo <= span.lo)
                    .checked_sub(1)
                    .map(|index| &assignments[index].provenance)
            })
            .or_else(|| self.parameter_aliases.get(&(scope, name.to_string())))
            .or(Some(declaration))
    }

    pub(in crate::plugins::analysis::mainjs::api_classifier) fn binding_key(
        &self,
        ident: &Ident,
    ) -> BindingKey {
        BindingKey {
            name: ident.sym.to_string(),
            scope: self
                .binding_with_scope_at(ident.sym.as_ref(), ident.span)
                .map(|(scope, _)| scope),
        }
    }

    pub(super) fn owner_at(&self, span: Span) -> usize {
        let mut scope = self.scope_at(span);
        loop {
            if self.scopes[scope].kind == ScopeKind::Function {
                return scope + 1;
            }
            let Some(parent) = self.scopes[scope].parent else {
                return 0;
            };
            scope = parent;
        }
    }

    pub(in crate::plugins::analysis::mainjs::api_classifier) fn rooted_member_chain(
        &self,
        member: &MemberExpr,
    ) -> Option<String> {
        let raw = member_chain(member)?;
        self.rooted_member_chain_from_raw(member, &raw)
    }

    pub(super) fn rooted_member_chain_from_raw(
        &self,
        member: &MemberExpr,
        raw: &str,
    ) -> Option<String> {
        for prefix_end in member_prefix_ends(raw) {
            let property = &raw[..prefix_end];
            let Some(assignments) = self.property_assignments.get(property) else {
                continue;
            };
            let prior_count =
                assignments.partition_point(|assignment| assignment.span.lo <= member.span.lo);
            if let Some(assignment) = assignments[..prior_count]
                .iter()
                .rev()
                .find(|assignment| contains(self.scopes[assignment.scope].span, member.span))
            {
                let target = assignment.target.as_ref()?;
                return Some(format!("{target}{}", &raw[prefix_end..]));
            }
        }
        let Some(root) = member_root_ident(member) else {
            return raw.starts_with("this.").then(|| raw.to_string());
        };
        let suffix = raw.strip_prefix(root.sym.as_ref())?;
        match self.binding_at(root.sym.as_ref(), root.span) {
            Some(BindingProvenance::ValueAlias { target }) => Some(format!("{target}{suffix}")),
            Some(
                BindingProvenance::Local
                | BindingProvenance::ModuleExport { .. }
                | BindingProvenance::ModuleNamespace { .. },
            ) => None,
            None => Some(raw.to_string()),
        }
    }

    pub(super) fn rooted_expr_chain(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::This(_) => Some("this".to_string()),
            Expr::Ident(ident) => match self.binding_at(ident.sym.as_ref(), ident.span) {
                Some(BindingProvenance::ValueAlias { target }) => Some(target.clone()),
                Some(_) => None,
                None => Some(ident.sym.to_string()),
            },
            Expr::Member(member) => self.rooted_member_chain(member),
            Expr::OptChain(chain) => match &*chain.base {
                OptChainBase::Member(member) => self.rooted_member_chain(member),
                OptChainBase::Call(call) => self.rooted_expr_chain(&call.callee),
            },
            Expr::Paren(paren) => self.rooted_expr_chain(&paren.expr),
            _ => None,
        }
    }

    fn binding_with_scope_at(&self, name: &str, span: Span) -> Option<(usize, &BindingProvenance)> {
        let mut scope = self.scope_at(span);
        loop {
            if let Some(binding) = self.scopes[scope].bindings.get(name) {
                return Some((scope, binding));
            }
            scope = self.scopes[scope].parent?;
        }
    }

    fn scope_at(&self, span: Span) -> usize {
        let position = self
            .scopes_by_start
            .partition_point(|index| self.scopes[*index].span.lo <= span.lo);
        let Some(mut scope) = position
            .checked_sub(1)
            .map(|index| self.scopes_by_start[index])
        else {
            return 0;
        };

        while !contains(self.scopes[scope].span, span) {
            let Some(parent) = self.scopes[scope].parent else {
                return 0;
            };
            scope = parent;
        }
        scope
    }
}
