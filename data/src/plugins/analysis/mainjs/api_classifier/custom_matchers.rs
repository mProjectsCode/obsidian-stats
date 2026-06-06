use std::collections::{BTreeMap, BTreeSet};

use swc_ecma_ast::{
    AssignExpr, AssignTarget, CallExpr, Callee, Expr, MemberExpr, Pat, SimpleAssignTarget,
    VarDeclarator,
};
use swc_ecma_visit::{Visit, VisitWith};

use super::context::ApiMatchContext;
use super::result::{ApiEvidence, ApiMatchKind};
use super::symbol_index::{expr_name, member_chain, member_prop_name};

pub(super) fn remote_dom_loading(context: &ApiMatchContext<'_>) -> Vec<ApiEvidence> {
    let Some(program) = context.program else {
        return Vec::new();
    };
    let mut visitor = DomFlowVisitor::default();
    program.visit_with(&mut visitor);
    visitor
        .remote_resource_appends
        .into_iter()
        .map(|tag| custom_evidence(format!("remote_dom_loading:{tag}")))
        .collect()
}

pub(super) fn remote_dom_script_injection(context: &ApiMatchContext<'_>) -> Vec<ApiEvidence> {
    let Some(program) = context.program else {
        return Vec::new();
    };
    let mut visitor = DomFlowVisitor::default();
    program.visit_with(&mut visitor);
    visitor
        .script_appends
        .into_iter()
        .map(|binding| custom_evidence(format!("remote_dom_script_injection:{binding}")))
        .collect()
}

pub(super) fn dom_file_input(context: &ApiMatchContext<'_>) -> Vec<ApiEvidence> {
    let Some(program) = context.program else {
        return Vec::new();
    };
    let mut visitor = DomFlowVisitor::default();
    program.visit_with(&mut visitor);
    visitor
        .file_inputs
        .into_iter()
        .map(|binding| custom_evidence(format!("dom_file_input:{binding}")))
        .collect()
}

pub(super) fn adapter_operation(context: &ApiMatchContext<'_>) -> Vec<ApiEvidence> {
    let Some(program) = context.program else {
        return Vec::new();
    };
    let mut visitor = AdapterFlowVisitor::default();
    program.visit_with(&mut visitor);
    visitor
        .operations
        .into_iter()
        .map(|operation| custom_evidence(format!("vault_adapter:{operation}")))
        .collect()
}

pub(super) fn metadata_cache_extraction(context: &ApiMatchContext<'_>) -> Vec<ApiEvidence> {
    let Some(program) = context.program else {
        return Vec::new();
    };
    let mut visitor = MetadataCacheFlowVisitor::default();
    program.visit_with(&mut visitor);
    visitor
        .properties
        .into_iter()
        .map(|property| custom_evidence(format!("metadata_cache:{property}")))
        .collect()
}

fn custom_evidence(symbol: String) -> ApiEvidence {
    ApiEvidence {
        kind: ApiMatchKind::CustomAst,
        symbol,
        count: 1,
    }
}

#[derive(Default)]
struct DomFlowVisitor {
    scripts: BTreeSet<String>,
    remote_scripts: BTreeSet<String>,
    remote_resources: BTreeMap<String, String>,
    remote_resource_bindings: BTreeSet<String>,
    remote_resource_appends: BTreeSet<String>,
    script_appends: BTreeSet<String>,
    inputs: BTreeSet<String>,
    file_inputs: BTreeSet<String>,
}

impl Visit for DomFlowVisitor {
    fn visit_var_declarator(&mut self, declarator: &VarDeclarator) {
        if let Some(binding) = binding_ident(&declarator.name)
            && let Some(init) = declarator.init.as_deref()
            && let Some(tag) = create_element_tag(init)
        {
            match tag.as_str() {
                "script" => {
                    self.scripts.insert(binding.clone());
                }
                "img" | "link" | "style" => {
                    self.remote_resources.insert(binding.clone(), tag);
                }
                "input" => {
                    self.inputs.insert(binding.clone());
                }
                _ => {}
            }
        }

        declarator.visit_children_with(self);
    }

    fn visit_assign_expr(&mut self, assign: &AssignExpr) {
        if let Some((object, property)) = assigned_member(&assign.left) {
            if property == "src"
                && self.scripts.contains(&object)
                && expr_contains_remote_url(&assign.right)
            {
                self.remote_scripts.insert(object.clone());
            }
            if matches!(property.as_str(), "src" | "href")
                && self.remote_resources.contains_key(&object)
                && expr_contains_remote_url(&assign.right)
            {
                self.remote_resource_bindings.insert(object.clone());
            }
            if property == "type"
                && self.inputs.contains(&object)
                && string_expr(&assign.right).as_deref() == Some("file")
            {
                self.file_inputs.insert(object);
            }
        }

        assign.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, call: &CallExpr) {
        if is_append_child_call(call)
            && let Some(binding) = ident_arg(call, 0)
        {
            if self.remote_scripts.contains(&binding) {
                self.script_appends.insert(binding.clone());
            }
            if self.remote_resource_bindings.contains(&binding)
                && let Some(tag) = self.remote_resources.get(&binding)
            {
                self.remote_resource_appends.insert(tag.clone());
            }
        }

        call.visit_children_with(self);
    }
}

#[derive(Default)]
struct AdapterFlowVisitor {
    adapters: BTreeSet<String>,
    operations: BTreeSet<String>,
}

impl Visit for AdapterFlowVisitor {
    fn visit_var_declarator(&mut self, declarator: &VarDeclarator) {
        if let Some(binding) = binding_ident(&declarator.name)
            && declarator
                .init
                .as_deref()
                .and_then(expr_name)
                .is_some_and(|name| {
                    matches!(
                        name.as_str(),
                        "this.app.vault.adapter" | "app.vault.adapter"
                    )
                })
        {
            self.adapters.insert(binding);
        }

        declarator.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, call: &CallExpr) {
        if let Some(member) = call_member_callee(call)
            && let Some(object) = expr_name(&member.obj)
            && self.adapters.contains(&object)
            && let Some(operation) = member_prop_name(&member.prop)
            && ADAPTER_OPS.contains(&operation.as_str())
        {
            self.operations.insert(operation);
        }

        call.visit_children_with(self);
    }
}

#[derive(Default)]
struct MetadataCacheFlowVisitor {
    caches: BTreeSet<String>,
    properties: BTreeSet<String>,
}

impl Visit for MetadataCacheFlowVisitor {
    fn visit_var_declarator(&mut self, declarator: &VarDeclarator) {
        if let Some(binding) = binding_ident(&declarator.name)
            && declarator
                .init
                .as_deref()
                .is_some_and(is_metadata_cache_call)
        {
            self.caches.insert(binding);
        }

        declarator.visit_children_with(self);
    }

    fn visit_member_expr(&mut self, member: &MemberExpr) {
        if let Some(object) = expr_name(&member.obj)
            && self.caches.contains(&object)
            && let Some(property) = member_prop_name(&member.prop)
            && METADATA_PROPS.contains(&property.as_str())
        {
            self.properties.insert(property);
        }

        member.visit_children_with(self);
    }
}

const ADAPTER_OPS: &[&str] = &[
    "read", "write", "append", "mkdir", "rmdir", "remove", "rename", "copy", "exists", "list",
    "stat",
];

const METADATA_PROPS: &[&str] = &[
    "tags",
    "links",
    "embeds",
    "blocks",
    "headings",
    "sections",
    "listItems",
];

fn binding_ident(pat: &Pat) -> Option<String> {
    match pat {
        Pat::Ident(ident) => Some(ident.id.sym.to_string()),
        _ => None,
    }
}

fn create_element_tag(expr: &Expr) -> Option<String> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let Some(member) = call_member_callee(call) else {
        return None;
    };
    if member_chain(member).as_deref() != Some("document.createElement") {
        return None;
    }
    string_arg(call, 0)
}

fn is_metadata_cache_call(expr: &Expr) -> bool {
    let Expr::Call(call) = expr else {
        return false;
    };
    call_member_callee(call)
        .and_then(member_chain)
        .is_some_and(|chain| {
            matches!(
                chain.as_str(),
                "this.app.metadataCache.getFileCache"
                    | "app.metadataCache.getFileCache"
                    | "this.app.metadataCache.getCache"
                    | "app.metadataCache.getCache"
            )
        })
}

fn is_append_child_call(call: &CallExpr) -> bool {
    call_member_callee(call)
        .and_then(member_chain)
        .is_some_and(|chain| {
            matches!(
                chain.as_str(),
                "document.body.appendChild"
                    | "document.head.appendChild"
                    | "document.documentElement.appendChild"
            )
        })
}

fn call_member_callee(call: &CallExpr) -> Option<&MemberExpr> {
    let Callee::Expr(callee) = &call.callee else {
        return None;
    };
    let Expr::Member(member) = &**callee else {
        return None;
    };
    Some(member)
}

fn assigned_member(target: &AssignTarget) -> Option<(String, String)> {
    let AssignTarget::Simple(SimpleAssignTarget::Member(member)) = target else {
        return None;
    };
    Some((expr_name(&member.obj)?, member_prop_name(&member.prop)?))
}

fn string_arg(call: &CallExpr, index: usize) -> Option<String> {
    string_expr(&call.args.get(index)?.expr)
}

fn ident_arg(call: &CallExpr, index: usize) -> Option<String> {
    match &*call.args.get(index)?.expr {
        Expr::Ident(ident) => Some(ident.sym.to_string()),
        _ => None,
    }
}

fn string_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(swc_ecma_ast::Lit::Str(value)) => Some(value.value.to_string_lossy().to_string()),
        Expr::Tpl(tpl) if tpl.exprs.is_empty() && tpl.quasis.len() == 1 => {
            tpl.quasis.first().map(|quasi| quasi.raw.to_string())
        }
        Expr::Paren(paren) => string_expr(&paren.expr),
        _ => None,
    }
}

fn expr_contains_remote_url(expr: &Expr) -> bool {
    string_expr(expr)
        .as_deref()
        .is_some_and(|value| value.starts_with("https://") || value.starts_with("http://"))
}
