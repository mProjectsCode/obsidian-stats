use swc_ecma_ast::{
    AssignTarget, CallExpr, Callee, Expr, MemberExpr, Pat, PropName, SimpleAssignTarget,
};

use super::super::symbol_index::{
    AliasInfo, SymbolCallProvenance, member_chain, member_prop_name, static_string,
};

pub(super) fn binding_ident(pat: &Pat) -> Option<&swc_ecma_ast::BindingIdent> {
    match pat {
        Pat::Ident(ident) => Some(ident),
        _ => None,
    }
}

pub(super) fn create_element_tag(expr: &Expr) -> Option<String> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let member = call_member_callee(call)?;
    if member_chain(member).as_deref() != Some("document.createElement") {
        return None;
    }
    string_arg(call, 0)
}

pub(super) fn prop_name(name: &PropName) -> Option<String> {
    match name {
        PropName::Ident(ident) => Some(ident.sym.to_string()),
        PropName::Str(value) => Some(value.value.to_string_lossy().to_string()),
        PropName::Computed(computed) => static_string(&computed.expr),
        PropName::Num(_) | PropName::BigInt(_) => None,
    }
}

pub(super) fn is_string_timer_call(call: &CallExpr, aliases: &AliasInfo) -> bool {
    if call
        .args
        .first()
        .and_then(|argument| static_string(&argument.expr))
        .is_none()
    {
        return false;
    }

    let Callee::Expr(callee) = &call.callee else {
        return false;
    };
    match &**callee {
        Expr::Ident(ident) => {
            matches!(ident.sym.as_ref(), "setTimeout" | "setInterval")
                && aliases.call_provenance(ident.sym.as_ref(), ident.span)
                    == SymbolCallProvenance::Global
        }
        Expr::Member(member) => aliases.rooted_member_chain(member).is_some_and(|chain| {
            matches!(
                chain.as_str(),
                "globalThis.setTimeout"
                    | "globalThis.setInterval"
                    | "window.setTimeout"
                    | "window.setInterval"
                    | "self.setTimeout"
                    | "self.setInterval"
            )
        }),
        _ => false,
    }
}

pub(super) fn is_metadata_cache_call(expr: &Expr) -> bool {
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

pub(super) fn is_append_child_call(call: &CallExpr) -> bool {
    call_member_callee(call)
        .and_then(member_chain)
        .is_some_and(|chain| {
            matches!(
                chain.as_str(),
                "document.body.appendChild"
                    | "document.head.appendChild"
                    | "document.documentElement.appendChild"
                    | "document.body.append"
                    | "document.head.append"
                    | "document.documentElement.append"
                    | "document.body.prepend"
                    | "document.head.prepend"
                    | "document.documentElement.prepend"
                    | "document.body.insertBefore"
                    | "document.head.insertBefore"
                    | "document.documentElement.insertBefore"
            )
        })
}

pub(super) fn call_member_callee(call: &CallExpr) -> Option<&MemberExpr> {
    let Callee::Expr(callee) = &call.callee else {
        return None;
    };
    let Expr::Member(member) = &**callee else {
        return None;
    };
    Some(member)
}

pub(super) fn assigned_ident(target: &AssignTarget) -> Option<&swc_ecma_ast::Ident> {
    let AssignTarget::Simple(SimpleAssignTarget::Ident(ident)) = target else {
        return None;
    };
    Some(&ident.id)
}

pub(super) fn assigned_member(target: &AssignTarget) -> Option<(&swc_ecma_ast::Ident, String)> {
    let AssignTarget::Simple(SimpleAssignTarget::Member(member)) = target else {
        return None;
    };
    Some((expr_ident(&member.obj)?, member_prop_name(&member.prop)?))
}

pub(super) fn string_arg(call: &CallExpr, index: usize) -> Option<String> {
    static_string(&call.args.get(index)?.expr)
}

pub(super) fn ident_arg(call: &CallExpr, index: usize) -> Option<&swc_ecma_ast::Ident> {
    expr_ident(&call.args.get(index)?.expr)
}

pub(super) fn expr_ident(expr: &Expr) -> Option<&swc_ecma_ast::Ident> {
    match expr {
        Expr::Ident(ident) => Some(ident),
        Expr::Paren(paren) => expr_ident(&paren.expr),
        _ => None,
    }
}

pub(super) fn expr_contains_remote_url(expr: &Expr) -> bool {
    static_string(expr)
        .as_deref()
        .is_some_and(|value| value.starts_with("https://") || value.starts_with("http://"))
}
