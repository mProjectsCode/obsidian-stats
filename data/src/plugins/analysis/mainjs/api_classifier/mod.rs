use std::collections::{BTreeMap, BTreeSet};

use swc_ecma_ast::Program;

mod catalog;
mod context;
mod custom_matchers;
mod result;
mod rule;
mod symbol_index;

pub(super) use catalog::{obsidian_api_rules, validate_catalog};
pub(in crate::plugins::analysis) use result::{ApiClassificationResult, ApiMatchKind};
pub(in crate::plugins::analysis) use rule::{ApiCategory, ApiSeverity, Confidence};

use result::{ApiCapability, ApiEvidence, Disclosure};
use rule::ApiRule;
use symbol_index::SymbolIndex;

#[cfg(test)]
use rule::{ApiCatalogError, ApiRuleBuildError};

pub(super) fn classify_api_usage(
    _source: &str,
    program: Option<&Program>,
    rules: &[ApiRule],
) -> ApiClassificationResult {
    let context = context::ApiMatchContext::new(program);
    let (symbol_index, argument_evidence) =
        SymbolIndex::collect_for_rules(program, &context.aliases, rules);
    let primitive_matches = rules
        .iter()
        .enumerate()
        .map(|(index, rule)| rule_match(rule, &symbol_index, &context, &argument_evidence[index]))
        .collect::<Vec<_>>();
    let mut result = ApiClassificationResult::default();
    let mut emitted_ids = BTreeSet::new();
    let mut emitted_owners = BTreeMap::<String, BTreeSet<usize>>::new();

    loop {
        let mut emitted_any = false;
        for (rule, primitive_match) in rules.iter().zip(&primitive_matches) {
            let correlation_owners = correlation_owners(rule, &emitted_owners);
            if emitted_ids.contains(&rule.id)
                || (rule.is_correlation() && correlation_owners.is_empty())
            {
                continue;
            }

            let mut evidence = primitive_match.evidence.clone();
            if evidence.is_empty() {
                if rule.matcher.is_empty() && rule.is_correlation() {
                    evidence = correlation_evidence(rule);
                } else {
                    continue;
                }
            }
            if evidence.is_empty()
                || distinct_evidence_count(&evidence) < rule.min_distinct_evidence
            {
                continue;
            }

            let owners = if rule.is_correlation() {
                correlation_owners
            } else {
                primitive_match.owners.clone()
            };
            emit_rule(
                rule,
                evidence,
                owners,
                &mut result,
                &mut emitted_ids,
                &mut emitted_owners,
            );
            emitted_any = true;
        }
        if !emitted_any {
            break;
        }
    }

    result
}

struct RuleMatch {
    evidence: Vec<ApiEvidence>,
    owners: BTreeSet<usize>,
}

fn rule_match(
    rule: &ApiRule,
    symbol_index: &SymbolIndex,
    context: &context::ApiMatchContext<'_>,
    argument_evidence: &[ApiEvidence],
) -> RuleMatch {
    let mut evidence = symbol_index.evidence_for(rule);
    let owners = symbol_index.owners_for_evidence(&evidence);
    evidence.extend_from_slice(argument_evidence);
    if context.program.is_some() {
        for matcher in &rule.matcher.custom_ast {
            evidence.extend((matcher.matcher)(context));
        }
    }
    evidence.truncate(rule.evidence_limit);
    RuleMatch { evidence, owners }
}

fn emit_rule(
    rule: &ApiRule,
    evidence: Vec<ApiEvidence>,
    owners: BTreeSet<usize>,
    result: &mut ApiClassificationResult,
    emitted_ids: &mut BTreeSet<String>,
    emitted_owners: &mut BTreeMap<String, BTreeSet<usize>>,
) {
    emitted_ids.insert(rule.id.clone());
    emitted_owners.insert(rule.id.clone(), owners.clone());
    result.capabilities.push(ApiCapability {
        id: rule.id.clone(),
        label: rule.label.clone(),
        category: rule.category,
        severity: rule.severity,
        confidence: rule.confidence,
        evidence,
    });

    for disclosure_id in &rule.implies {
        emitted_ids.insert(disclosure_id.clone());
        emitted_owners
            .entry(disclosure_id.clone())
            .or_default()
            .extend(owners.iter().copied());
        result.disclosures.push(Disclosure {
            id: disclosure_id.clone(),
            from_capability: rule.id.clone(),
        });
    }
}

fn correlation_owners(
    rule: &ApiRule,
    emitted_owners: &BTreeMap<String, BTreeSet<usize>>,
) -> BTreeSet<usize> {
    if !rule.is_correlation() {
        return BTreeSet::new();
    }

    let mut required = rule.when_all.iter();
    let mut owners = if let Some(first) = required.next() {
        let Some(owners) = emitted_owners.get(first) else {
            return BTreeSet::new();
        };
        owners.clone()
    } else {
        emitted_owners
            .values()
            .flat_map(|owners| owners.iter().copied())
            .collect()
    };
    for dependency in required {
        let Some(dependency_owners) = emitted_owners.get(dependency) else {
            return BTreeSet::new();
        };
        owners.retain(|owner| dependency_owners.contains(owner));
    }

    if !rule.when_any.is_empty() {
        let any_owners = rule
            .when_any
            .iter()
            .filter_map(|dependency| emitted_owners.get(dependency))
            .flat_map(|owners| owners.iter().copied())
            .collect::<BTreeSet<_>>();
        owners.retain(|owner| any_owners.contains(owner));
    }
    owners
}

fn correlation_evidence(rule: &ApiRule) -> Vec<ApiEvidence> {
    rule.when_all
        .iter()
        .chain(&rule.when_any)
        .take(rule.evidence_limit)
        .map(|dependency| ApiEvidence {
            kind: ApiMatchKind::Correlation,
            symbol: dependency.clone(),
            count: 1,
        })
        .collect()
}

fn distinct_evidence_count(evidence: &[ApiEvidence]) -> usize {
    evidence
        .iter()
        .map(|evidence| (evidence.kind, evidence.symbol.as_str()))
        .collect::<BTreeSet<_>>()
        .len()
}

#[cfg(test)]
mod tests;
