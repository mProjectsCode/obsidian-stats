use std::collections::{BTreeMap, BTreeSet};

use swc_ecma_ast::Program;

use super::result::{ApiEvidence, ApiMatchKind};
use super::rule::{ApiRule, CallMatcher, CallProvenance, MemberCallMatcher, MemberCallProvenance};

mod alias;
mod ast;
mod visitor;

pub(super) use alias::{AliasInfo, BindingKey};
pub(super) use ast::{SymbolCallProvenance, expr_name, member_chain, member_prop_name};

#[derive(Debug, Default)]
pub(super) struct SymbolIndex {
    calls: BTreeMap<String, u32>,
    global_calls: BTreeMap<String, u32>,
    module_calls: BTreeMap<(String, String), u32>,
    member_calls: BTreeMap<String, u32>,
    rooted_member_calls: BTreeMap<String, u32>,
    module_member_calls: BTreeMap<(String, String), u32>,
    member_reads: BTreeMap<String, u32>,
    imports: BTreeMap<String, u32>,
    string_literals: BTreeMap<String, u32>,
    classes: BTreeMap<String, u32>,
    constructors: BTreeMap<String, u32>,
    owners: BTreeMap<(ApiMatchKind, String), Vec<usize>>,
}

impl SymbolIndex {
    pub(super) fn collect(program: Option<&Program>, aliases: &AliasInfo) -> Self {
        Self::collect_with_argument_matchers(program, aliases, &[], 0).0
    }

    pub(super) fn collect_for_rules(
        program: Option<&Program>,
        aliases: &AliasInfo,
        rules: &[ApiRule],
    ) -> (Self, Vec<Vec<ApiEvidence>>) {
        let matchers = rules
            .iter()
            .enumerate()
            .flat_map(|(rule_index, rule)| {
                rule.matcher
                    .member_calls
                    .iter()
                    .filter(|matcher| !matcher.arg_strings.is_empty())
                    .cloned()
                    .map(move |matcher| (rule_index, matcher))
            })
            .collect::<Vec<_>>();
        Self::collect_with_argument_matchers(program, aliases, &matchers, rules.len())
    }

    fn collect_with_argument_matchers(
        program: Option<&Program>,
        aliases: &AliasInfo,
        argument_matchers: &[(usize, MemberCallMatcher)],
        rule_count: usize,
    ) -> (Self, Vec<Vec<ApiEvidence>>) {
        let mut index = Self::default();
        let mut argument_evidence = vec![Vec::new(); rule_count];
        if let Some(program) = program {
            visitor::collect(
                program,
                aliases,
                argument_matchers,
                &mut index,
                &mut argument_evidence,
            );
        }

        (index, argument_evidence)
    }

    pub(super) fn evidence_for(&self, rule: &ApiRule) -> Vec<ApiEvidence> {
        let mut evidence = Vec::new();
        self.collect_call_evidence(&rule.matcher.calls, &mut evidence);
        self.collect_member_call_evidence(&rule.matcher.member_calls, &mut evidence);
        self.collect_member_read_evidence(&rule.matcher.member_reads, &mut evidence);
        self.collect_evidence(ApiMatchKind::Import, &rule.matcher.imports, &mut evidence);
        self.collect_string_literal_evidence(&rule.matcher.string_literals, &mut evidence);
        self.collect_evidence(ApiMatchKind::Class, &rule.matcher.classes, &mut evidence);
        self.collect_evidence(
            ApiMatchKind::Constructor,
            &rule.matcher.constructors,
            &mut evidence,
        );

        evidence.truncate(rule.evidence_limit);
        evidence
    }

    pub(super) fn owners_for_evidence(&self, evidence: &[ApiEvidence]) -> BTreeSet<usize> {
        evidence
            .iter()
            .flat_map(|evidence| {
                if evidence.kind == ApiMatchKind::StringLiteral {
                    self.owners
                        .iter()
                        .filter(move |((kind, literal), _)| {
                            *kind == ApiMatchKind::StringLiteral
                                && literal.contains(&evidence.symbol)
                        })
                        .flat_map(|(_, owners)| owners.iter().copied())
                        .collect::<Vec<_>>()
                } else if evidence.kind == ApiMatchKind::MemberRead
                    && !evidence.symbol.contains('.')
                {
                    let suffix = format!(".{}", evidence.symbol);
                    self.owners
                        .iter()
                        .filter(move |((kind, symbol), _)| {
                            *kind == ApiMatchKind::MemberRead
                                && (symbol == &evidence.symbol || symbol.ends_with(&suffix))
                        })
                        .flat_map(|(_, owners)| owners.iter().copied())
                        .collect::<Vec<_>>()
                } else {
                    self.owners
                        .get(&(evidence.kind, evidence.symbol.clone()))
                        .into_iter()
                        .flat_map(|owners| owners.iter().copied())
                        .collect()
                }
            })
            .collect()
    }

    fn collect_call_evidence(&self, calls: &[CallMatcher], evidence: &mut Vec<ApiEvidence>) {
        for call in calls {
            let count = match &call.provenance {
                CallProvenance::Any => self.calls.get(&call.name).copied(),
                CallProvenance::Global => self.global_calls.get(&call.name).copied(),
                CallProvenance::ModuleExport { module } => self
                    .module_calls
                    .get(&(module.clone(), call.name.clone()))
                    .copied(),
            };

            if let Some(count) = count {
                evidence.push(ApiEvidence {
                    kind: ApiMatchKind::Call,
                    symbol: call.evidence_symbol(),
                    count,
                });
            }
        }
    }

    fn collect_member_read_evidence(
        &self,
        member_reads: &[String],
        evidence: &mut Vec<ApiEvidence>,
    ) {
        for symbol in member_reads {
            let count = if symbol.contains('.') {
                self.member_reads.get(symbol).copied().unwrap_or(0)
            } else {
                let suffix = format!(".{symbol}");
                self.member_reads
                    .iter()
                    .fold(0u32, |count, (member_read, member_read_count)| {
                        if member_read == symbol || member_read.ends_with(&suffix) {
                            count.saturating_add(*member_read_count)
                        } else {
                            count
                        }
                    })
            };

            if count > 0 {
                evidence.push(ApiEvidence {
                    kind: ApiMatchKind::MemberRead,
                    symbol: symbol.clone(),
                    count,
                });
            }
        }
    }

    fn collect_member_call_evidence(
        &self,
        member_calls: &[MemberCallMatcher],
        evidence: &mut Vec<ApiEvidence>,
    ) {
        for call in member_calls {
            if !call.arg_strings.is_empty() {
                continue;
            }
            let count = match &call.provenance {
                MemberCallProvenance::Any => self.member_calls.get(&call.chain).copied(),
                MemberCallProvenance::Rooted => self.rooted_member_calls.get(&call.chain).copied(),
                MemberCallProvenance::ModuleNamespace { module } => self
                    .module_member_calls
                    .get(&(module.clone(), call.chain.clone()))
                    .copied(),
            };

            if let Some(count) = count {
                evidence.push(ApiEvidence {
                    kind: ApiMatchKind::MemberCall,
                    symbol: call.evidence_symbol(),
                    count,
                });
            }
        }
    }

    fn collect_evidence(
        &self,
        kind: ApiMatchKind,
        symbols: &[String],
        evidence: &mut Vec<ApiEvidence>,
    ) {
        for symbol in symbols {
            if let Some(count) = self.count(kind, symbol) {
                evidence.push(ApiEvidence {
                    kind,
                    symbol: symbol.clone(),
                    count,
                });
            }
        }
    }

    fn collect_string_literal_evidence(&self, markers: &[String], evidence: &mut Vec<ApiEvidence>) {
        for marker in markers {
            let count =
                self.string_literals
                    .iter()
                    .fold(0u32, |count, (literal, literal_count)| {
                        if literal.contains(marker) {
                            count.saturating_add(*literal_count)
                        } else {
                            count
                        }
                    });

            if count > 0 {
                evidence.push(ApiEvidence {
                    kind: ApiMatchKind::StringLiteral,
                    symbol: marker.clone(),
                    count,
                });
            }
        }
    }

    fn count(&self, kind: ApiMatchKind, symbol: &str) -> Option<u32> {
        match kind {
            ApiMatchKind::Call => self.calls.get(symbol),
            ApiMatchKind::MemberCall => self.member_calls.get(symbol),
            ApiMatchKind::MemberRead => self.member_reads.get(symbol),
            ApiMatchKind::Import => self.imports.get(symbol),
            ApiMatchKind::StringLiteral => self.string_literals.get(symbol),
            ApiMatchKind::Class => self.classes.get(symbol),
            ApiMatchKind::Constructor => self.constructors.get(symbol),
            ApiMatchKind::CallArgument | ApiMatchKind::CustomAst => None,
            ApiMatchKind::Correlation => None,
        }
        .copied()
    }

    fn increment(&mut self, kind: ApiMatchKind, symbol: impl Into<String>) {
        let target = match kind {
            ApiMatchKind::Call => &mut self.calls,
            ApiMatchKind::MemberCall => &mut self.member_calls,
            ApiMatchKind::MemberRead => &mut self.member_reads,
            ApiMatchKind::Import => &mut self.imports,
            ApiMatchKind::StringLiteral => &mut self.string_literals,
            ApiMatchKind::Class => &mut self.classes,
            ApiMatchKind::Constructor => &mut self.constructors,
            ApiMatchKind::CallArgument | ApiMatchKind::CustomAst => return,
            ApiMatchKind::Correlation => return,
        };

        *target.entry(symbol.into()).or_insert(0) += 1;
    }

    fn record_owner(&mut self, kind: ApiMatchKind, symbol: impl Into<String>, owner: usize) {
        let owners = self.owners.entry((kind, symbol.into())).or_default();
        if !owners.contains(&owner) {
            owners.push(owner);
        }
    }
}
