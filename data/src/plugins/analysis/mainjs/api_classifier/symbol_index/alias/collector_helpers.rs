use swc_common::Span;
use swc_ecma_ast::{ObjectPatProp, Pat};

use super::super::ast::prop_name;
use super::{BindingProvenance, collector::AliasCollector};

pub(super) fn collect_value_aliases(
    pat: &Pat,
    target: &str,
    scope: usize,
    aliases: &mut AliasCollector,
) {
    match pat {
        Pat::Ident(ident) => aliases.insert(
            scope,
            ident.id.sym.to_string(),
            BindingProvenance::ValueAlias {
                target: target.to_string(),
            },
        ),
        Pat::Object(object) => {
            for prop in &object.props {
                match prop {
                    ObjectPatProp::KeyValue(key_value) => {
                        if let Some(property) = prop_name(&key_value.key) {
                            collect_value_aliases(
                                &key_value.value,
                                &format!("{target}.{property}"),
                                scope,
                                aliases,
                            );
                        }
                    }
                    ObjectPatProp::Assign(assign) => {
                        let property = assign.key.sym.to_string();
                        aliases.insert(
                            scope,
                            property.clone(),
                            BindingProvenance::ValueAlias {
                                target: format!("{target}.{property}"),
                            },
                        );
                    }
                    ObjectPatProp::Rest(_) => {}
                }
            }
        }
        _ => {}
    }
}

pub(super) fn collect_assignment_aliases(
    pat: &Pat,
    target: &str,
    span: Span,
    scope: usize,
    aliases: &mut AliasCollector,
) {
    match pat {
        Pat::Ident(ident) => aliases.record_assignment(
            span,
            scope,
            ident.id.sym.to_string(),
            BindingProvenance::ValueAlias {
                target: target.to_string(),
            },
        ),
        Pat::Object(object) => {
            for property in &object.props {
                match property {
                    ObjectPatProp::KeyValue(key_value) => {
                        if let Some(name) = prop_name(&key_value.key) {
                            collect_assignment_aliases(
                                &key_value.value,
                                &format!("{target}.{name}"),
                                span,
                                scope,
                                aliases,
                            );
                        }
                    }
                    ObjectPatProp::Assign(assign) => {
                        let name = assign.key.sym.to_string();
                        aliases.record_assignment(
                            span,
                            scope,
                            name.clone(),
                            BindingProvenance::ValueAlias {
                                target: format!("{target}.{name}"),
                            },
                        );
                    }
                    ObjectPatProp::Rest(_) => {}
                }
            }
        }
        Pat::Assign(assign) => {
            collect_assignment_aliases(&assign.left, target, span, scope, aliases);
        }
        _ => {}
    }
}

pub(super) fn contains(outer: Span, inner: Span) -> bool {
    outer.lo <= inner.lo && outer.hi >= inner.hi
}

pub(super) fn member_prefix_ends(chain: &str) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(chain.len()).chain(chain.rmatch_indices('.').map(|(index, _)| index))
}

pub(super) fn collect_require_aliases(
    pat: &Pat,
    module: String,
    scope: usize,
    aliases: &mut AliasCollector,
) {
    match pat {
        Pat::Ident(ident) => {
            aliases.insert(
                scope,
                ident.id.sym.to_string(),
                BindingProvenance::ModuleNamespace { module },
            );
        }
        Pat::Object(object) => {
            for prop in &object.props {
                match prop {
                    ObjectPatProp::KeyValue(key_value) => {
                        if let Some(imported) = prop_name(&key_value.key) {
                            collect_require_export_alias(
                                &key_value.value,
                                &module,
                                &imported,
                                scope,
                                aliases,
                            );
                        }
                    }
                    ObjectPatProp::Assign(assign) => {
                        let local = assign.key.sym.to_string();
                        aliases.insert(
                            scope,
                            local.clone(),
                            BindingProvenance::ModuleExport {
                                module: module.clone(),
                                export: local,
                            },
                        );
                    }
                    ObjectPatProp::Rest(_) => {}
                }
            }
        }
        _ => {}
    }
}

fn collect_require_export_alias(
    pat: &Pat,
    module: &str,
    export: &str,
    scope: usize,
    aliases: &mut AliasCollector,
) {
    if let Pat::Ident(local) = pat {
        aliases.insert(
            scope,
            local.id.sym.to_string(),
            BindingProvenance::ModuleExport {
                module: module.to_string(),
                export: export.to_string(),
            },
        );
    }
}
