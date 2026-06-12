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
use rule::ApiRule;
use symbol_index::SymbolIndex;

#[cfg(test)]
use rule::{ApiCatalogError, ApiCategory, ApiRuleBuildError};

pub(super) fn classify_api_usage(
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

    result
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
