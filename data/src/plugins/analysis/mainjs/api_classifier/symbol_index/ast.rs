use std::collections::BTreeSet;

use swc_ecma_ast::{
    BinaryOp, CallExpr, Callee, Expr, Ident, Lit, MemberExpr, MemberProp, ModuleExportName,
    ObjectPatProp, OptChainBase, Pat,
};

pub(super) fn member_root_ident(member: &MemberExpr) -> Option<&Ident> {
    expr_root_ident(&member.obj)
}

fn expr_root_ident(expr: &Expr) -> Option<&Ident> {
    match expr {
        Expr::Ident(ident) => Some(ident),
        Expr::Member(parent) => member_root_ident(parent),
        Expr::OptChain(chain) => match &*chain.base {
            OptChainBase::Member(member) => member_root_ident(member),
            OptChainBase::Call(call) => expr_root_ident(&call.callee),
        },
        Expr::Paren(paren) => expr_root_ident(&paren.expr),
        _ => None,
    }
}

pub(super) fn expr_member(expr: &Expr) -> Option<&MemberExpr> {
    match expr {
        Expr::Member(member) => Some(member),
        Expr::OptChain(chain) => match &*chain.base {
            OptChainBase::Member(member) => Some(member),
            OptChainBase::Call(call) => expr_member(&call.callee),
        },
        Expr::Paren(paren) => expr_member(&paren.expr),
        _ => None,
    }
}

pub(super) fn effective_callee_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Paren(paren) => effective_callee_expr(&paren.expr),
        Expr::Seq(sequence) => sequence
            .exprs
            .last()
            .map_or(expr, |expr| effective_callee_expr(expr)),
        _ => expr,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::plugins::analysis::mainjs::api_classifier) enum SymbolCallProvenance {
    Global,
    Local,
    ModuleExport { module: String, export: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::plugins::analysis::mainjs::api_classifier) enum SymbolMemberProvenance {
    ModuleNamespace { module: String, member: String },
}

pub(super) fn collect_pat_bindings(pat: &Pat, bindings: &mut BTreeSet<String>) {
    match pat {
        Pat::Ident(ident) => {
            bindings.insert(ident.id.sym.to_string());
        }
        Pat::Array(array) => {
            for elem in array.elems.iter().flatten() {
                collect_pat_bindings(elem, bindings);
            }
        }
        Pat::Rest(rest) => collect_pat_bindings(&rest.arg, bindings),
        Pat::Object(object) => {
            for prop in &object.props {
                match prop {
                    ObjectPatProp::KeyValue(key_value) => {
                        collect_pat_bindings(&key_value.value, bindings);
                    }
                    ObjectPatProp::Assign(assign) => {
                        bindings.insert(assign.key.sym.to_string());
                    }
                    ObjectPatProp::Rest(rest) => collect_pat_bindings(&rest.arg, bindings),
                }
            }
        }
        Pat::Assign(assign) => collect_pat_bindings(&assign.left, bindings),
        Pat::Invalid(_) | Pat::Expr(_) => {}
    }
}

pub(super) fn binding_ident_name(pat: &Pat) -> Option<String> {
    match pat {
        Pat::Ident(ident) => Some(ident.id.sym.to_string()),
        Pat::Assign(assign) => binding_ident_name(&assign.left),
        _ => None,
    }
}

pub(in crate::plugins::analysis::mainjs::api_classifier) fn require_module_name(
    expr: &Expr,
) -> Option<String> {
    match expr {
        Expr::Call(call) => require_call_module_name(call).or_else(|| {
            let Callee::Expr(callee) = &call.callee else {
                return None;
            };
            let Expr::Ident(wrapper) = &**callee else {
                return None;
            };
            is_module_interop_wrapper(wrapper.sym.as_ref())
                .then(|| call.args.first())
                .flatten()
                .and_then(|arg| require_module_name(&arg.expr))
        }),
        Expr::Member(member) => require_module_name(&member.obj),
        Expr::Paren(paren) => require_module_name(&paren.expr),
        _ => None,
    }
}

fn is_module_interop_wrapper(name: &str) -> bool {
    matches!(
        name,
        "__toESM"
            | "__importStar"
            | "__importDefault"
            | "_interopRequireWildcard"
            | "_interopRequireDefault"
    )
}

pub(super) fn require_call_module_name(call: &CallExpr) -> Option<String> {
    let Callee::Expr(callee) = &call.callee else {
        return None;
    };
    let Expr::Ident(ident) = &**callee else {
        return None;
    };
    if ident.sym != *"require" {
        return None;
    }

    call.args.first().and_then(|arg| match &*arg.expr {
        Expr::Lit(Lit::Str(value)) => Some(value.value.to_string_lossy().to_string()),
        _ => None,
    })
}

pub(super) fn module_export_name(name: &ModuleExportName) -> String {
    match name {
        ModuleExportName::Ident(ident) => ident.sym.to_string(),
        ModuleExportName::Str(value) => value.value.to_string_lossy().to_string(),
    }
}

pub(super) fn prop_name(name: &swc_ecma_ast::PropName) -> Option<String> {
    match name {
        swc_ecma_ast::PropName::Ident(ident) => Some(ident.sym.to_string()),
        swc_ecma_ast::PropName::Str(value) => Some(value.value.to_string_lossy().to_string()),
        swc_ecma_ast::PropName::Num(number) => Some(number.value.to_string()),
        swc_ecma_ast::PropName::Computed(computed) => {
            if let Expr::Lit(Lit::Str(value)) = &*computed.expr {
                Some(value.value.to_string_lossy().to_string())
            } else {
                None
            }
        }
        swc_ecma_ast::PropName::BigInt(_) => None,
    }
}

pub(in crate::plugins::analysis::mainjs::api_classifier) fn expr_name(
    expr: &Expr,
) -> Option<String> {
    match expr {
        Expr::Ident(ident) => Some(ident.sym.to_string()),
        Expr::Member(member) => member_chain(member),
        Expr::This(_) => Some("this".to_string()),
        Expr::OptChain(chain) => match &*chain.base {
            OptChainBase::Member(member) => member_chain(member),
            OptChainBase::Call(call) => expr_name(&call.callee),
        },
        Expr::Paren(paren) => expr_name(&paren.expr),
        Expr::TsAs(expr) => expr_name(&expr.expr),
        Expr::TsNonNull(expr) => expr_name(&expr.expr),
        Expr::TsSatisfies(expr) => expr_name(&expr.expr),
        Expr::TsTypeAssertion(expr) => expr_name(&expr.expr),
        _ => None,
    }
}

pub(in crate::plugins::analysis::mainjs::api_classifier) fn member_chain(
    member: &MemberExpr,
) -> Option<String> {
    let object = expr_name(&member.obj)?;
    let prop = member_prop_name(&member.prop)?;
    Some(format!("{object}.{prop}"))
}

pub(in crate::plugins::analysis::mainjs::api_classifier) fn member_prop_name(
    prop: &MemberProp,
) -> Option<String> {
    match prop {
        MemberProp::Ident(ident) => Some(ident.sym.to_string()),
        MemberProp::PrivateName(name) => Some(format!("#{}", name.name)),
        MemberProp::Computed(computed) => static_property_name(&computed.expr),
    }
}

fn static_property_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(Lit::Str(value)) => Some(value.value.to_string_lossy().to_string()),
        Expr::Lit(Lit::Num(value)) => Some(value.value.to_string()),
        Expr::Tpl(template) if template.exprs.is_empty() && template.quasis.len() == 1 => {
            template.quasis.first().map(|quasi| quasi.raw.to_string())
        }
        Expr::Bin(binary) if binary.op == BinaryOp::Add => Some(format!(
            "{}{}",
            static_property_name(&binary.left)?,
            static_property_name(&binary.right)?
        )),
        Expr::Paren(paren) => static_property_name(&paren.expr),
        _ => None,
    }
}

pub(in crate::plugins::analysis::mainjs::api_classifier) fn static_string(
    expr: &Expr,
) -> Option<String> {
    match expr {
        Expr::Lit(Lit::Str(value)) => Some(value.value.to_string_lossy().to_string()),
        Expr::Tpl(template) if template.exprs.is_empty() && template.quasis.len() == 1 => {
            template.quasis.first().map(|quasi| quasi.raw.to_string())
        }
        Expr::Paren(paren) => static_string(&paren.expr),
        _ => None,
    }
}
