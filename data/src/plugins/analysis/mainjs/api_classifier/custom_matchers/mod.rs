use std::collections::BTreeSet;

use super::context::ApiMatchContext;
use super::result::{ApiEvidence, ApiMatchKind};
use super::symbol_index::{AliasInfo, BindingKey};

mod ast;
mod visitor;

#[derive(Debug, Default)]
pub(super) struct SemanticIndex {
    remote_resource_appends: BTreeSet<String>,
    script_appends: BTreeSet<BindingKey>,
    file_inputs: BTreeSet<BindingKey>,
    metadata_properties: BTreeSet<String>,
    dynamic_code_sites: BTreeSet<String>,
    lifecycle_methods: BTreeSet<String>,
}

impl SemanticIndex {
    pub(super) fn collect(program: &swc_ecma_ast::Program, aliases: &AliasInfo) -> Self {
        visitor::collect(program, aliases)
    }
}

pub(super) fn remote_dom_loading(context: &ApiMatchContext<'_>) -> Vec<ApiEvidence> {
    custom_evidence(
        &context.semantics.remote_resource_appends,
        "remote_dom_loading",
    )
}

pub(super) fn remote_dom_script_injection(context: &ApiMatchContext<'_>) -> Vec<ApiEvidence> {
    custom_evidence(
        &context.semantics.script_appends,
        "remote_dom_script_injection",
    )
}

pub(super) fn dom_file_input(context: &ApiMatchContext<'_>) -> Vec<ApiEvidence> {
    custom_evidence(&context.semantics.file_inputs, "dom_file_input")
}

pub(super) fn metadata_cache_extraction(context: &ApiMatchContext<'_>) -> Vec<ApiEvidence> {
    custom_evidence(&context.semantics.metadata_properties, "metadata_cache")
}

pub(super) fn dynamic_code_execution(context: &ApiMatchContext<'_>) -> Vec<ApiEvidence> {
    custom_evidence(&context.semantics.dynamic_code_sites, "dynamic_code")
}

pub(super) fn lifecycle_methods(context: &ApiMatchContext<'_>) -> Vec<ApiEvidence> {
    custom_evidence(&context.semantics.lifecycle_methods, "lifecycle_method")
}

fn custom_evidence<T: std::fmt::Display + Ord>(
    values: &BTreeSet<T>,
    prefix: &str,
) -> Vec<ApiEvidence> {
    values
        .iter()
        .map(|value| ApiEvidence {
            kind: ApiMatchKind::CustomAst,
            symbol: format!("{prefix}:{value}"),
            count: 1,
        })
        .collect()
}
