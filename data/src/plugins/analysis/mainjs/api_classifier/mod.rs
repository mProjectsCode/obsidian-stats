use swc_ecma_ast::Program;

mod catalog;
mod context;
mod custom_matchers;
mod result;
mod rule;
mod symbol_index;

pub(super) use catalog::{obsidian_api_rules, validate_catalog};
pub(in crate::plugins::analysis) use result::ApiClassificationResult;
pub(in crate::plugins::analysis) use rule::{ApiSeverity, Confidence};

use result::{ApiCapability, ApiEvidence, Disclosure};
use rule::{ApiCategory, ApiRule};
use symbol_index::SymbolIndex;

#[cfg(test)]
use rule::{ApiCatalogError, ApiRuleBuildError};

#[cfg(test)]
pub(super) fn classify_api_usage(
    program: Option<&Program>,
    rules: &[ApiRule],
) -> ApiClassificationResult {
    classify_api_usage_with_source("", program, rules)
}

pub(super) fn classify_api_usage_with_source(
    source: &str,
    program: Option<&Program>,
    rules: &[ApiRule],
) -> ApiClassificationResult {
    let context = context::ApiMatchContext::new(program);
    let (symbol_index, argument_evidence) =
        SymbolIndex::collect_for_rules(program, &context.aliases, rules);
    let mut result = ApiClassificationResult::default();

    for (index, rule) in rules.iter().enumerate() {
        let evidence = rule_match(rule, &symbol_index, &context, &argument_evidence[index]);
        if !evidence.is_empty() {
            emit_rule(rule, evidence, &mut result);
        }
    }

    emit_bundle_findings(source, program, &mut result);

    result
}

fn emit_bundle_findings(
    source: &str,
    program: Option<&Program>,
    result: &mut ApiClassificationResult,
) {
    if super::check_sourcemap::detect_sourcemap_comment(source) {
        emit_static_finding(
            "bundle.source_map_comment",
            "Includes a source map comment",
            ApiCategory::Bundle,
            "sourceMappingURL",
            1,
            "disclosure.source_map_comment",
            result,
        );
    }

    let (base64_count, _) = super::check_base64::detect_base64(source);
    if base64_count > 0 {
        emit_static_finding(
            "bundle.embedded_base64_blob",
            "Includes large embedded base64 blobs",
            ApiCategory::Bundle,
            "base64 blob",
            base64_count,
            "disclosure.embedded_base64_blob",
            result,
        );
    }

    let worker_count = super::check_worker::detect_worker_usage(source, program);
    if worker_count > 0 {
        emit_static_finding(
            "browser.worker",
            "Uses Worker APIs",
            ApiCategory::Browser,
            "Worker",
            worker_count,
            "disclosure.worker_usage",
            result,
        );
    }

    let webassembly_count = super::check_wasm::detect_webassembly_usage(source, program);
    if webassembly_count > 0 {
        emit_static_finding(
            "browser.webassembly",
            "Uses WebAssembly APIs",
            ApiCategory::Browser,
            "WebAssembly",
            webassembly_count,
            "disclosure.webassembly_usage",
            result,
        );
    }
}

fn emit_static_finding(
    id: &str,
    label: &str,
    category: ApiCategory,
    symbol: &str,
    count: u32,
    disclosure_id: &str,
    result: &mut ApiClassificationResult,
) {
    result.capabilities.push(ApiCapability {
        id: id.to_string(),
        label: label.to_string(),
        category,
        severity: ApiSeverity::Info,
        confidence: Confidence::High,
        evidence: vec![ApiEvidence {
            kind: result::ApiMatchKind::CustomAst,
            symbol: symbol.to_string(),
            count,
        }],
    });
    result.disclosures.push(Disclosure {
        id: disclosure_id.to_string(),
        from_capability: id.to_string(),
    });
}

fn rule_match(
    rule: &ApiRule,
    symbol_index: &SymbolIndex,
    context: &context::ApiMatchContext<'_>,
    argument_evidence: &[ApiEvidence],
) -> Vec<ApiEvidence> {
    let mut evidence = symbol_index.evidence_for(rule);
    evidence.extend_from_slice(argument_evidence);
    if context.program.is_some() {
        for matcher in &rule.matcher.custom_ast {
            evidence.extend((matcher.matcher)(context));
        }
    }
    evidence.truncate(ApiRule::EVIDENCE_LIMIT);
    evidence
}

fn emit_rule(rule: &ApiRule, evidence: Vec<ApiEvidence>, result: &mut ApiClassificationResult) {
    result.capabilities.push(ApiCapability {
        id: rule.id.clone(),
        label: rule.label.clone(),
        category: rule.category,
        severity: rule.severity,
        confidence: rule.confidence,
        evidence,
    });

    for disclosure_id in &rule.implies {
        result.disclosures.push(Disclosure {
            id: disclosure_id.clone(),
            from_capability: rule.id.clone(),
        });
    }
}

#[cfg(test)]
mod tests;
