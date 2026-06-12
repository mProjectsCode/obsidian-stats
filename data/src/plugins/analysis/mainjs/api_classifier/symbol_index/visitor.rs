use std::collections::BTreeMap;

use swc_ecma_ast::{
    CallExpr, Callee, Expr, Ident, ImportDecl, MemberExpr, NewExpr, OptChainBase, OptChainExpr,
    Program, Str, Tpl,
};
use swc_ecma_visit::{Visit, VisitWith};

use super::alias::AliasInfo;
use super::ast::{
    SymbolCallProvenance, SymbolMemberProvenance, effective_callee_expr, expr_member, expr_name,
    member_chain, require_call_module_name, static_string,
};
use super::{ApiEvidence, ApiMatchKind, MemberCallMatcher, MemberCallProvenance, SymbolIndex};
use crate::plugins::analysis::mainjs::api_classifier::rule::canonical_rooted_chain;

pub(super) fn collect(
    program: &Program,
    aliases: &AliasInfo,
    argument_matchers: &[(usize, MemberCallMatcher)],
    index: &mut SymbolIndex,
    argument_evidence: &mut [Vec<ApiEvidence>],
) {
    let mut visitor = SymbolIndexVisitor {
        index,
        aliases,
        pending_callee_reads: BTreeMap::new(),
        argument_matchers,
        argument_evidence,
    };
    program.visit_with(&mut visitor);
}

struct SymbolIndexVisitor<'a, 'rules> {
    index: &'a mut SymbolIndex,
    aliases: &'a AliasInfo,
    pending_callee_reads: BTreeMap<String, u32>,
    argument_matchers: &'rules [(usize, MemberCallMatcher)],
    argument_evidence: &'a mut [Vec<ApiEvidence>],
}

impl SymbolIndexVisitor<'_, '_> {
    fn record_identifier_call(&mut self, ident: &Ident) {
        let name = ident.sym.to_string();
        self.index.increment(ApiMatchKind::Call, name.clone());

        match self.aliases.call_provenance(&name, ident.span) {
            SymbolCallProvenance::Global => {
                *self.index.global_calls.entry(name).or_insert(0) += 1;
            }
            SymbolCallProvenance::ModuleExport { module, export } => {
                *self
                    .index
                    .module_calls
                    .entry((module.clone(), export.clone()))
                    .or_insert(0) += 1;
            }
            SymbolCallProvenance::Local => {}
        }
    }

    fn record_member_call(&mut self, member: &MemberExpr, call: Option<&CallExpr>) {
        let syntactic_chain = member_chain(member);
        let resolved_chain = syntactic_chain
            .as_deref()
            .and_then(|chain| self.aliases.resolve_member_chain(member, chain));
        let module_member = syntactic_chain
            .as_deref()
            .and_then(|chain| self.aliases.member_call_provenance_for_chain(member, chain));

        if let Some(call) = call {
            self.collect_argument_evidence(
                call,
                syntactic_chain.as_deref(),
                resolved_chain.as_deref(),
                module_member.as_ref(),
            );
        }
        if let Some(chain) = syntactic_chain {
            self.index
                .increment(ApiMatchKind::MemberCall, chain.clone());
            *self.pending_callee_reads.entry(chain.clone()).or_insert(0) += 1;
        }
        if let Some(chain) = resolved_chain {
            let chain = canonical_rooted_chain(&chain).to_string();
            *self
                .index
                .rooted_member_calls
                .entry(chain.clone())
                .or_insert(0) += 1;
        }
        if let Some(SymbolMemberProvenance::ModuleNamespace { module, member }) = module_member {
            *self
                .index
                .module_member_calls
                .entry((module.clone(), member.clone()))
                .or_insert(0) += 1;
        }
    }

    fn collect_argument_evidence(
        &mut self,
        call: &CallExpr,
        syntactic_chain: Option<&str>,
        resolved_chain: Option<&str>,
        module_member: Option<&SymbolMemberProvenance>,
    ) {
        for (rule_index, matcher) in self.argument_matchers {
            let member_matches = match &matcher.provenance {
                MemberCallProvenance::Any => syntactic_chain == Some(&matcher.chain),
                MemberCallProvenance::Rooted => resolved_chain
                    .map(canonical_rooted_chain)
                    .is_some_and(|chain| chain == matcher.chain),
                MemberCallProvenance::ModuleNamespace { module } => matches!(
                    module_member,
                    Some(SymbolMemberProvenance::ModuleNamespace {
                        module: found_module,
                        member
                    }) if found_module == module && member == &matcher.chain
                ),
            };
            if member_matches
                && matcher.arg_strings.iter().all(|arg_matcher| {
                    call.args
                        .get(arg_matcher.index)
                        .and_then(|argument| static_string(&argument.expr))
                        .is_some_and(|value| {
                            arg_matcher.values.iter().any(|expected| expected == &value)
                        })
                })
            {
                self.argument_evidence[*rule_index].push(ApiEvidence {
                    kind: ApiMatchKind::CallArgument,
                    symbol: matcher.evidence_symbol(),
                    count: 1,
                });
            }
        }
    }
}

impl Visit for SymbolIndexVisitor<'_, '_> {
    fn visit_import_decl(&mut self, import: &ImportDecl) {
        let module = import.src.value.to_string_lossy().to_string();
        self.index.increment(ApiMatchKind::Import, module);
    }

    fn visit_call_expr(&mut self, call: &CallExpr) {
        if let Some(module) = require_call_module_name(call) {
            self.index.increment(ApiMatchKind::Import, module);
        }

        match &call.callee {
            Callee::Expr(callee) => match effective_callee_expr(callee) {
                Expr::Ident(ident) => {
                    self.record_identifier_call(ident);
                }
                Expr::Member(member) => {
                    self.record_member_call(member, Some(call));
                }
                _ => {}
            },
            Callee::Super(_) => self.index.increment(ApiMatchKind::Call, "super"),
            Callee::Import(_) => self.index.increment(ApiMatchKind::Call, "import"),
        }

        call.visit_children_with(self);
    }

    fn visit_opt_chain_expr(&mut self, chain: &OptChainExpr) {
        if let OptChainBase::Call(call) = &*chain.base {
            match &*call.callee {
                Expr::Ident(ident) => self.record_identifier_call(ident),
                Expr::Member(member) => self.record_member_call(member, None),
                _ => {
                    if let Some(raw) = expr_name(&call.callee) {
                        self.index.increment(ApiMatchKind::MemberCall, raw);
                    }
                    if let Some(rooted) = self.aliases.rooted_expr_chain(&call.callee) {
                        let rooted = canonical_rooted_chain(&rooted).to_string();
                        *self
                            .index
                            .rooted_member_calls
                            .entry(rooted.clone())
                            .or_insert(0) += 1;
                    }
                    if let Some(member) = expr_member(&call.callee)
                        && let Some(SymbolMemberProvenance::ModuleNamespace { module, member }) =
                            self.aliases.member_call_provenance(member)
                    {
                        *self
                            .index
                            .module_member_calls
                            .entry((module.clone(), member.clone()))
                            .or_insert(0) += 1;
                    }
                }
            }
        }
        chain.visit_children_with(self);
    }

    fn visit_member_expr(&mut self, member: &MemberExpr) {
        let syntactic_chain = member_chain(member);
        if let Some(chain) = syntactic_chain.as_ref() {
            if let Some(skip_count) = self.pending_callee_reads.get_mut(chain.as_str()) {
                *skip_count -= 1;
                if *skip_count == 0 {
                    self.pending_callee_reads.remove(chain.as_str());
                }

                member.visit_children_with(self);
                return;
            }

            let chain = canonical_rooted_chain(chain).to_string();
            self.index.increment(ApiMatchKind::MemberRead, chain);
        }
        let module_member = syntactic_chain
            .as_deref()
            .and_then(|chain| self.aliases.member_call_provenance_for_chain(member, chain));
        if let Some(SymbolMemberProvenance::ModuleNamespace {
            module,
            member: class,
        }) = module_member
            && module == "obsidian"
        {
            self.index.increment(ApiMatchKind::Class, class);
        }

        member.visit_children_with(self);
    }

    fn visit_new_expr(&mut self, new_expr: &NewExpr) {
        match &*new_expr.callee {
            Expr::Ident(ident) => {
                match self.aliases.call_provenance(ident.sym.as_ref(), ident.span) {
                    SymbolCallProvenance::Global => {
                        self.index
                            .increment(ApiMatchKind::Constructor, ident.sym.to_string());
                    }
                    SymbolCallProvenance::ModuleExport { module, export } => {
                        self.index
                            .increment(ApiMatchKind::Constructor, export.clone());
                        self.index
                            .increment(ApiMatchKind::Constructor, format!("{module}.{export}"));
                    }
                    SymbolCallProvenance::Local => {}
                }
            }
            Expr::Member(member) => {
                if let Some(SymbolMemberProvenance::ModuleNamespace { module, member }) =
                    self.aliases.member_call_provenance(member)
                {
                    self.index
                        .increment(ApiMatchKind::Constructor, member.clone());
                    self.index
                        .increment(ApiMatchKind::Constructor, format!("{module}.{member}"));
                }
            }
            _ => {}
        }

        new_expr.visit_children_with(self);
    }

    fn visit_ident(&mut self, ident: &Ident) {
        if let SymbolCallProvenance::ModuleExport { module, export } =
            self.aliases.call_provenance(ident.sym.as_ref(), ident.span)
            && module == "obsidian"
        {
            self.index.increment(ApiMatchKind::Class, export);
        }
    }

    fn visit_str(&mut self, value: &Str) {
        let literal = value.value.to_string_lossy().to_string();
        self.index.increment(ApiMatchKind::StringLiteral, literal);

        value.visit_children_with(self);
    }

    fn visit_tpl(&mut self, template: &Tpl) {
        for quasi in &template.quasis {
            let literal = quasi
                .cooked
                .as_ref()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| quasi.raw.to_string());
            self.index.increment(ApiMatchKind::StringLiteral, literal);
        }

        template.visit_children_with(self);
    }
}
