use std::collections::BTreeMap;

use swc_ecma_ast::{
    CallExpr, Callee, Expr, Ident, ImportDecl, MemberExpr, NewExpr, OptChainBase, OptChainExpr,
    Program, Str,
};
use swc_ecma_visit::{Visit, VisitWith};

use super::alias::AliasInfo;
use super::ast::{
    SymbolCallProvenance, SymbolMemberProvenance, expr_member, expr_name, literal_string,
    member_chain, require_call_module_name,
};
use super::{ApiEvidence, ApiMatchKind, MemberCallMatcher, MemberCallProvenance, SymbolIndex};

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
        callee_member_reads_to_skip: BTreeMap::new(),
        argument_matchers,
        argument_evidence,
    };
    program.visit_with(&mut visitor);
}

struct SymbolIndexVisitor<'a, 'rules> {
    index: &'a mut SymbolIndex,
    aliases: &'a AliasInfo,
    callee_member_reads_to_skip: BTreeMap<String, u32>,
    argument_matchers: &'rules [(usize, MemberCallMatcher)],
    argument_evidence: &'a mut [Vec<ApiEvidence>],
}

impl SymbolIndexVisitor<'_, '_> {
    fn collect_argument_evidence(
        &mut self,
        call: &CallExpr,
        raw_chain: Option<&str>,
        rooted_chain: Option<&str>,
        module_member: Option<&SymbolMemberProvenance>,
    ) {
        for (rule_index, matcher) in self.argument_matchers {
            let member_matches = match &matcher.provenance {
                MemberCallProvenance::Any => raw_chain == Some(&matcher.chain),
                MemberCallProvenance::Rooted => rooted_chain == Some(&matcher.chain),
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
                        .and_then(|argument| literal_string(&argument.expr))
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
        self.index.increment(ApiMatchKind::Import, module.clone());
        self.index.record_owner(ApiMatchKind::Import, module, 0);
    }

    fn visit_call_expr(&mut self, call: &CallExpr) {
        if let Some(module) = require_call_module_name(call) {
            self.index.increment(ApiMatchKind::Import, module);
        }

        match &call.callee {
            Callee::Expr(callee) => match &**callee {
                Expr::Ident(ident) => {
                    let name = ident.sym.to_string();
                    self.index.increment(ApiMatchKind::Call, name.clone());
                    let owner = self.aliases.owner_at(ident.span);
                    self.index
                        .record_owner(ApiMatchKind::Call, name.clone(), owner);
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
                            self.index.record_owner(
                                ApiMatchKind::Call,
                                format!("{module}.{export}"),
                                owner,
                            );
                        }
                        SymbolCallProvenance::Local => {}
                    }
                }
                Expr::Member(member) => {
                    let owner = self.aliases.owner_at(member.span);
                    let raw_chain = member_chain(member);
                    let rooted_chain = raw_chain
                        .as_deref()
                        .and_then(|chain| self.aliases.rooted_member_chain_from_raw(member, chain));
                    let module_member = raw_chain.as_deref().and_then(|chain| {
                        self.aliases.member_call_provenance_from_raw(member, chain)
                    });
                    self.collect_argument_evidence(
                        call,
                        raw_chain.as_deref(),
                        rooted_chain.as_deref(),
                        module_member.as_ref(),
                    );
                    if let Some(chain) = raw_chain {
                        self.index
                            .increment(ApiMatchKind::MemberCall, chain.clone());
                        if let Some(SymbolMemberProvenance::ModuleNamespace { module, member }) =
                            &module_member
                        {
                            *self
                                .index
                                .module_member_calls
                                .entry((module.clone(), member.clone()))
                                .or_insert(0) += 1;
                        }
                        *self
                            .callee_member_reads_to_skip
                            .entry(chain.clone())
                            .or_insert(0) += 1;
                        self.index
                            .record_owner(ApiMatchKind::MemberCall, chain, owner);
                    }
                    if let Some(chain) = rooted_chain {
                        *self
                            .index
                            .rooted_member_calls
                            .entry(chain.clone())
                            .or_insert(0) += 1;
                        self.index
                            .record_owner(ApiMatchKind::MemberCall, chain, owner);
                    }
                    if let Some(SymbolMemberProvenance::ModuleNamespace { module, member }) =
                        module_member
                    {
                        self.index.record_owner(
                            ApiMatchKind::MemberCall,
                            format!("{module}.{member}"),
                            owner,
                        );
                    }
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
            let owner = self.aliases.owner_at(chain.span);
            match &*call.callee {
                Expr::Ident(ident) => {
                    let name = ident.sym.to_string();
                    self.index.increment(ApiMatchKind::Call, name.clone());
                    self.index
                        .record_owner(ApiMatchKind::Call, name.clone(), owner);
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
                            self.index.record_owner(
                                ApiMatchKind::Call,
                                format!("{module}.{export}"),
                                owner,
                            );
                        }
                        SymbolCallProvenance::Local => {}
                    }
                }
                Expr::Member(member) => {
                    if let Some(raw) = member_chain(member) {
                        self.index.increment(ApiMatchKind::MemberCall, raw.clone());
                        *self
                            .callee_member_reads_to_skip
                            .entry(raw.clone())
                            .or_insert(0) += 1;
                        self.index
                            .record_owner(ApiMatchKind::MemberCall, raw, owner);
                    }
                    if let Some(rooted) = self.aliases.rooted_member_chain(member) {
                        *self
                            .index
                            .rooted_member_calls
                            .entry(rooted.clone())
                            .or_insert(0) += 1;
                        self.index
                            .record_owner(ApiMatchKind::MemberCall, rooted, owner);
                    }
                    if let Some(SymbolMemberProvenance::ModuleNamespace { module, member }) =
                        self.aliases.member_call_provenance(member)
                    {
                        *self
                            .index
                            .module_member_calls
                            .entry((module.clone(), member.clone()))
                            .or_insert(0) += 1;
                        self.index.record_owner(
                            ApiMatchKind::MemberCall,
                            format!("{module}.{member}"),
                            owner,
                        );
                    }
                }
                _ => {
                    if let Some(raw) = expr_name(&call.callee) {
                        self.index.increment(ApiMatchKind::MemberCall, raw.clone());
                        self.index
                            .record_owner(ApiMatchKind::MemberCall, raw, owner);
                    }
                    if let Some(rooted) = self.aliases.rooted_expr_chain(&call.callee) {
                        *self
                            .index
                            .rooted_member_calls
                            .entry(rooted.clone())
                            .or_insert(0) += 1;
                        self.index
                            .record_owner(ApiMatchKind::MemberCall, rooted, owner);
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
                        self.index.record_owner(
                            ApiMatchKind::MemberCall,
                            format!("{module}.{member}"),
                            owner,
                        );
                    }
                }
            }
        }
        chain.visit_children_with(self);
    }

    fn visit_member_expr(&mut self, member: &MemberExpr) {
        let raw_chain = member_chain(member);
        if let Some(chain) = raw_chain.as_ref() {
            if let Some(skip_count) = self.callee_member_reads_to_skip.get_mut(chain.as_str()) {
                *skip_count -= 1;
                if *skip_count == 0 {
                    self.callee_member_reads_to_skip.remove(chain.as_str());
                }

                member.visit_children_with(self);
                return;
            }

            self.index
                .increment(ApiMatchKind::MemberRead, chain.clone());
            self.index.record_owner(
                ApiMatchKind::MemberRead,
                chain,
                self.aliases.owner_at(member.span),
            );
        }
        let module_member = raw_chain
            .as_deref()
            .and_then(|chain| self.aliases.member_call_provenance_from_raw(member, chain));
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
                        self.index.record_owner(
                            ApiMatchKind::Constructor,
                            ident.sym.to_string(),
                            self.aliases.owner_at(new_expr.span),
                        );
                    }
                    SymbolCallProvenance::ModuleExport { module, export } => {
                        self.index
                            .increment(ApiMatchKind::Constructor, export.clone());
                        self.index.record_owner(
                            ApiMatchKind::Constructor,
                            export.clone(),
                            self.aliases.owner_at(new_expr.span),
                        );
                        self.index
                            .increment(ApiMatchKind::Constructor, format!("{module}.{export}"));
                        self.index.record_owner(
                            ApiMatchKind::Constructor,
                            format!("{module}.{export}"),
                            self.aliases.owner_at(new_expr.span),
                        );
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
                    self.index.record_owner(
                        ApiMatchKind::Constructor,
                        member.clone(),
                        self.aliases.owner_at(new_expr.span),
                    );
                    self.index
                        .increment(ApiMatchKind::Constructor, format!("{module}.{member}"));
                    self.index.record_owner(
                        ApiMatchKind::Constructor,
                        format!("{module}.{member}"),
                        self.aliases.owner_at(new_expr.span),
                    );
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
            self.index.increment(ApiMatchKind::Class, export.clone());
            self.index.record_owner(
                ApiMatchKind::Class,
                export,
                self.aliases.owner_at(ident.span),
            );
        }
    }

    fn visit_str(&mut self, value: &Str) {
        let literal = value.value.to_string_lossy().to_string();
        self.index
            .increment(ApiMatchKind::StringLiteral, literal.clone());
        self.index.record_owner(
            ApiMatchKind::StringLiteral,
            literal,
            self.aliases.owner_at(value.span),
        );

        value.visit_children_with(self);
    }
}
