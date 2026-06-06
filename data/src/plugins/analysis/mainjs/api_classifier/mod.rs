use std::collections::BTreeSet;

use swc_ecma_ast::{CallExpr, Callee, Expr, Program};
use swc_ecma_visit::{Visit, VisitWith};

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
use rule::{ApiRule, MemberCallMatcher};
use symbol_index::SymbolIndex;

#[cfg(test)]
use rule::{ApiCatalogError, ApiRuleBuildError};

pub(super) fn classify_api_usage(
    source: &str,
    program: Option<&Program>,
    rules: &[ApiRule],
) -> ApiClassificationResult {
    let context = context::ApiMatchContext::new(source, program);
    let symbol_index = SymbolIndex::collect(source, program, rules);
    let mut result = ApiClassificationResult::default();
    let mut emitted_ids = BTreeSet::new();

    for rule in rules {
        if !correlation_matches(rule, &emitted_ids) {
            continue;
        }

        let mut evidence = rule_evidence(rule, &symbol_index, &context);
        if evidence.is_empty() {
            if rule.matcher.is_empty() && rule.is_correlation() {
                evidence = correlation_evidence(rule);
            } else {
                continue;
            }
        }
        if evidence.is_empty() {
            continue;
        }
        if distinct_evidence_count(&evidence) < rule.min_distinct_evidence {
            continue;
        }

        emit_rule(rule, evidence, &mut result, &mut emitted_ids);
    }

    result
}

fn rule_evidence(
    rule: &ApiRule,
    symbol_index: &SymbolIndex,
    context: &context::ApiMatchContext<'_>,
) -> Vec<ApiEvidence> {
    let mut evidence = symbol_index.evidence_for(rule);
    evidence.extend(argument_member_call_evidence(rule, context));
    if context.program.is_some() {
        for matcher in &rule.matcher.custom_ast {
            evidence.extend((matcher.matcher)(context));
        }
    }
    evidence.truncate(rule.evidence_limit);
    evidence
}

fn argument_member_call_evidence(
    rule: &ApiRule,
    context: &context::ApiMatchContext<'_>,
) -> Vec<ApiEvidence> {
    let Some(program) = context.program else {
        return Vec::new();
    };
    let matchers = rule
        .matcher
        .member_calls
        .iter()
        .filter(|matcher| !matcher.arg_strings.is_empty())
        .collect::<Vec<_>>();
    if matchers.is_empty() {
        return Vec::new();
    }

    let mut visitor = ArgumentMemberCallVisitor {
        context,
        matchers,
        evidence: Vec::new(),
    };
    program.visit_with(&mut visitor);
    visitor.evidence
}

struct ArgumentMemberCallVisitor<'a> {
    context: &'a context::ApiMatchContext<'a>,
    matchers: Vec<&'a MemberCallMatcher>,
    evidence: Vec<ApiEvidence>,
}

impl Visit for ArgumentMemberCallVisitor<'_> {
    fn visit_call_expr(&mut self, call: &CallExpr) {
        let Some(member) = call_member_callee(call) else {
            call.visit_children_with(self);
            return;
        };

        for matcher in &self.matchers {
            if !self
                .context
                .is_member_call_match(member, &matcher.chain, &matcher.provenance)
            {
                continue;
            }
            if matcher.arg_strings.iter().all(|arg_matcher| {
                self.context
                    .literal_arg(call, arg_matcher.index)
                    .is_some_and(|value| {
                        arg_matcher.values.iter().any(|expected| expected == &value)
                    })
            }) {
                self.evidence.push(ApiEvidence {
                    kind: ApiMatchKind::CallArgument,
                    symbol: matcher.evidence_symbol(),
                    count: 1,
                });
            }
        }

        call.visit_children_with(self);
    }
}

fn call_member_callee(call: &CallExpr) -> Option<&swc_ecma_ast::MemberExpr> {
    let Callee::Expr(callee) = &call.callee else {
        return None;
    };
    let Expr::Member(member) = &**callee else {
        return None;
    };
    Some(member)
}

fn emit_rule(
    rule: &ApiRule,
    evidence: Vec<ApiEvidence>,
    result: &mut ApiClassificationResult,
    emitted_ids: &mut BTreeSet<String>,
) {
    emitted_ids.insert(rule.id.clone());
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
        result.disclosures.push(Disclosure {
            id: disclosure_id.clone(),
            from_capability: rule.id.clone(),
        });
    }
}

fn correlation_matches(rule: &ApiRule, emitted_ids: &BTreeSet<String>) -> bool {
    rule.when_all
        .iter()
        .all(|dependency| emitted_ids.contains(dependency))
        && (rule.when_any.is_empty()
            || rule
                .when_any
                .iter()
                .any(|dependency| emitted_ids.contains(dependency)))
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
