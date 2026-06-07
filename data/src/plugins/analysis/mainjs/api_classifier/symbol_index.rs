use std::collections::{BTreeMap, BTreeSet};

use swc_common::{Span, Spanned};
use swc_ecma_ast::{
    ArrowExpr, AssignExpr, AssignTarget, BinaryOp, BlockStmt, CallExpr, Callee, CatchClause,
    ClassDecl, Expr, FnDecl, Function, Ident, ImportDecl, ImportSpecifier, Lit, MemberExpr,
    MemberProp, ModuleExportName, NewExpr, ObjectPatProp, OptChainBase, OptChainExpr, Pat, Program,
    SimpleAssignTarget, Str, VarDecl, VarDeclKind,
};
use swc_ecma_visit::{Visit, VisitWith};

use super::result::{ApiEvidence, ApiMatchKind};
use super::rule::{ApiRule, CallMatcher, CallProvenance, MemberCallMatcher, MemberCallProvenance};

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
            let mut visitor = SymbolIndexVisitor {
                index: &mut index,
                aliases,
                callee_member_reads_to_skip: BTreeMap::new(),
                argument_matchers,
                argument_evidence: &mut argument_evidence,
            };
            program.visit_with(&mut visitor);
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

#[derive(Debug, Default, Clone)]
pub(super) struct AliasInfo {
    scopes: Vec<AliasScope>,
    scopes_by_start: Vec<usize>,
    assignments: BTreeMap<usize, BTreeMap<String, Vec<AliasAssignment>>>,
    property_assignments: BTreeMap<String, Vec<PropertyAliasAssignment>>,
    parameter_aliases: BTreeMap<(usize, String), BindingProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct BindingKey {
    name: String,
    scope: Option<usize>,
}

impl std::fmt::Display for BindingKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(formatter)
    }
}

#[derive(Debug, Clone)]
struct AliasScope {
    span: Span,
    depth: usize,
    kind: ScopeKind,
    parent: Option<usize>,
    bindings: BTreeMap<String, BindingProvenance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeKind {
    Program,
    Function,
    Block,
}

#[derive(Debug, Clone)]
enum BindingProvenance {
    Local,
    ValueAlias { target: String },
    ModuleExport { module: String, export: String },
    ModuleNamespace { module: String },
}

#[derive(Debug, Clone)]
struct AliasAssignment {
    span: Span,
    scope: usize,
    name: String,
    provenance: BindingProvenance,
}

#[derive(Debug, Clone)]
struct PropertyAliasAssignment {
    span: Span,
    scope: usize,
    property: String,
    target: Option<String>,
}

impl AliasInfo {
    pub(super) fn collect(program: &Program) -> Self {
        let mut collector = AliasCollector::new(program.span());
        program.visit_children_with(&mut collector);
        let parameter_aliases = collector.parameter_aliases();
        let mut scopes_by_start = (0..collector.scopes.len()).collect::<Vec<_>>();
        scopes_by_start.sort_by_key(|index| {
            let scope = &collector.scopes[*index];
            (scope.span.lo, scope.depth)
        });
        let mut assignments = BTreeMap::<usize, BTreeMap<String, Vec<AliasAssignment>>>::new();
        for assignment in collector.assignments {
            assignments
                .entry(assignment.scope)
                .or_default()
                .entry(assignment.name.clone())
                .or_default()
                .push(assignment);
        }
        for scope_assignments in assignments.values_mut() {
            for binding_assignments in scope_assignments.values_mut() {
                binding_assignments.sort_by_key(|assignment| assignment.span.lo);
            }
        }
        let mut property_assignments = BTreeMap::<String, Vec<PropertyAliasAssignment>>::new();
        for assignment in collector.property_assignments {
            property_assignments
                .entry(assignment.property.clone())
                .or_default()
                .push(assignment);
        }
        for assignments in property_assignments.values_mut() {
            assignments.sort_by_key(|assignment| assignment.span.lo);
        }
        Self {
            scopes: collector.scopes,
            scopes_by_start,
            assignments,
            property_assignments,
            parameter_aliases,
        }
    }

    pub(super) fn call_provenance(&self, name: &str, span: Span) -> SymbolCallProvenance {
        match self.binding_at(name, span) {
            Some(BindingProvenance::ModuleExport { module, export }) => {
                SymbolCallProvenance::ModuleExport {
                    module: module.clone(),
                    export: export.clone(),
                }
            }
            Some(
                BindingProvenance::Local
                | BindingProvenance::ValueAlias { .. }
                | BindingProvenance::ModuleNamespace { .. },
            ) => SymbolCallProvenance::Local,
            None => SymbolCallProvenance::Global,
        }
    }

    pub(super) fn member_call_provenance(
        &self,
        member: &MemberExpr,
    ) -> Option<SymbolMemberProvenance> {
        let chain = member_chain(member)?;
        self.member_call_provenance_from_raw(member, &chain)
    }

    fn member_call_provenance_from_raw(
        &self,
        member: &MemberExpr,
        chain: &str,
    ) -> Option<SymbolMemberProvenance> {
        let root = member_root_ident(member)?;
        let member = chain.strip_prefix(root.sym.as_ref())?.strip_prefix('.')?;
        match self.binding_at(root.sym.as_ref(), root.span) {
            Some(BindingProvenance::ModuleNamespace { module }) => {
                Some(SymbolMemberProvenance::ModuleNamespace {
                    module: module.clone(),
                    member: member.to_string(),
                })
            }
            _ => None,
        }
    }

    fn binding_at(&self, name: &str, span: Span) -> Option<&BindingProvenance> {
        let (scope, declaration) = self.binding_with_scope_at(name, span)?;
        self.assignments
            .get(&scope)
            .and_then(|assignments| assignments.get(name))
            .and_then(|assignments| {
                assignments
                    .partition_point(|assignment| assignment.span.lo <= span.lo)
                    .checked_sub(1)
                    .map(|index| &assignments[index].provenance)
            })
            .or_else(|| self.parameter_aliases.get(&(scope, name.to_string())))
            .or(Some(declaration))
    }

    pub(super) fn binding_key(&self, ident: &Ident) -> BindingKey {
        BindingKey {
            name: ident.sym.to_string(),
            scope: self
                .binding_with_scope_at(ident.sym.as_ref(), ident.span)
                .map(|(scope, _)| scope),
        }
    }

    pub(super) fn owner_at(&self, span: Span) -> usize {
        let mut scope = self.scope_at(span);
        loop {
            if self.scopes[scope].kind == ScopeKind::Function {
                return scope + 1;
            }
            let Some(parent) = self.scopes[scope].parent else {
                return 0;
            };
            scope = parent;
        }
    }

    pub(super) fn rooted_member_chain(&self, member: &MemberExpr) -> Option<String> {
        let raw = member_chain(member)?;
        self.rooted_member_chain_from_raw(member, &raw)
    }

    fn rooted_member_chain_from_raw(&self, member: &MemberExpr, raw: &str) -> Option<String> {
        for prefix_end in member_prefix_ends(raw) {
            let property = &raw[..prefix_end];
            let Some(assignments) = self.property_assignments.get(property) else {
                continue;
            };
            let prior_count =
                assignments.partition_point(|assignment| assignment.span.lo <= member.span.lo);
            if let Some(assignment) = assignments[..prior_count]
                .iter()
                .rev()
                .find(|assignment| contains(self.scopes[assignment.scope].span, member.span))
            {
                let target = assignment.target.as_ref()?;
                return Some(format!("{target}{}", &raw[prefix_end..]));
            }
        }
        let Some(root) = member_root_ident(member) else {
            return raw.starts_with("this.").then(|| raw.to_string());
        };
        let suffix = raw.strip_prefix(root.sym.as_ref())?;
        match self.binding_at(root.sym.as_ref(), root.span) {
            Some(BindingProvenance::ValueAlias { target }) => Some(format!("{target}{suffix}")),
            Some(
                BindingProvenance::Local
                | BindingProvenance::ModuleExport { .. }
                | BindingProvenance::ModuleNamespace { .. },
            ) => None,
            None => Some(raw.to_string()),
        }
    }

    fn rooted_expr_chain(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::This(_) => Some("this".to_string()),
            Expr::Ident(ident) => match self.binding_at(ident.sym.as_ref(), ident.span) {
                Some(BindingProvenance::ValueAlias { target }) => Some(target.clone()),
                Some(_) => None,
                None => Some(ident.sym.to_string()),
            },
            Expr::Member(member) => self.rooted_member_chain(member),
            Expr::OptChain(chain) => match &*chain.base {
                OptChainBase::Member(member) => self.rooted_member_chain(member),
                OptChainBase::Call(call) => self.rooted_expr_chain(&call.callee),
            },
            Expr::Paren(paren) => self.rooted_expr_chain(&paren.expr),
            _ => None,
        }
    }

    fn binding_with_scope_at(&self, name: &str, span: Span) -> Option<(usize, &BindingProvenance)> {
        let mut scope = self.scope_at(span);
        loop {
            if let Some(binding) = self.scopes[scope].bindings.get(name) {
                return Some((scope, binding));
            }
            scope = self.scopes[scope].parent?;
        }
    }

    fn scope_at(&self, span: Span) -> usize {
        let position = self
            .scopes_by_start
            .partition_point(|index| self.scopes[*index].span.lo <= span.lo);
        let Some(mut scope) = position
            .checked_sub(1)
            .map(|index| self.scopes_by_start[index])
        else {
            return 0;
        };

        while !contains(self.scopes[scope].span, span) {
            let Some(parent) = self.scopes[scope].parent else {
                return 0;
            };
            scope = parent;
        }
        scope
    }
}

struct AliasCollector {
    scopes: Vec<AliasScope>,
    stack: Vec<usize>,
    assignments: Vec<AliasAssignment>,
    latest_assignments: BTreeMap<usize, BTreeMap<String, BindingProvenance>>,
    property_assignments: Vec<PropertyAliasAssignment>,
    functions: BTreeMap<String, (usize, Vec<String>)>,
    calls: Vec<(String, Vec<Option<String>>)>,
}

impl AliasCollector {
    fn new(program_span: Span) -> Self {
        Self {
            scopes: vec![AliasScope {
                span: program_span,
                depth: 0,
                kind: ScopeKind::Program,
                parent: None,
                bindings: BTreeMap::new(),
            }],
            stack: vec![0],
            assignments: Vec::new(),
            latest_assignments: BTreeMap::new(),
            property_assignments: Vec::new(),
            functions: BTreeMap::new(),
            calls: Vec::new(),
        }
    }

    fn current_scope(&self) -> usize {
        *self.stack.last().expect("program scope is always present")
    }

    fn binding_scope(&self, kind: VarDeclKind) -> usize {
        if kind != VarDeclKind::Var {
            return self.current_scope();
        }
        self.stack
            .iter()
            .rev()
            .copied()
            .find(|index| {
                matches!(
                    self.scopes[*index].kind,
                    ScopeKind::Program | ScopeKind::Function
                )
            })
            .expect("program scope is always present")
    }

    fn insert(&mut self, scope: usize, name: impl Into<String>, provenance: BindingProvenance) {
        self.scopes[scope].bindings.insert(name.into(), provenance);
    }

    fn insert_local(&mut self, scope: usize, name: impl Into<String>) {
        self.insert(scope, name, BindingProvenance::Local);
    }

    fn record_assignment(
        &mut self,
        span: Span,
        scope: usize,
        name: String,
        provenance: BindingProvenance,
    ) {
        self.latest_assignments
            .entry(scope)
            .or_default()
            .insert(name.clone(), provenance.clone());
        self.assignments.push(AliasAssignment {
            span,
            scope,
            name,
            provenance,
        });
    }

    fn push_scope(&mut self, span: Span, kind: ScopeKind) {
        let index = self.scopes.len();
        let parent = self.current_scope();
        self.scopes.push(AliasScope {
            span,
            depth: self.stack.len(),
            kind,
            parent: Some(parent),
            bindings: BTreeMap::new(),
        });
        self.stack.push(index);
    }

    fn pop_scope(&mut self) {
        self.stack.pop();
    }

    fn insert_pat_locals(&mut self, scope: usize, pat: &Pat) {
        let mut bindings = BTreeSet::new();
        collect_pat_bindings(pat, &mut bindings);
        for binding in bindings {
            self.insert_local(scope, binding);
        }
    }

    fn visible_binding(&self, name: &str) -> Option<&BindingProvenance> {
        for scope in self.stack.iter().rev().copied() {
            if let Some(assignment) = self
                .latest_assignments
                .get(&scope)
                .and_then(|assignments| assignments.get(name))
            {
                return Some(assignment);
            }
            if let Some(binding) = self.scopes[scope].bindings.get(name) {
                return Some(binding);
            }
        }
        None
    }

    fn rooted_expr_name(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::This(_) => Some("this".to_string()),
            Expr::Ident(ident) => match self.visible_binding(ident.sym.as_ref()) {
                Some(BindingProvenance::ValueAlias { target }) => Some(target.clone()),
                Some(_) => None,
                None => Some(ident.sym.to_string()),
            },
            Expr::Member(member) => {
                let object = self.rooted_expr_name(&member.obj)?;
                let property = member_prop_name(&member.prop)?;
                Some(format!("{object}.{property}"))
            }
            Expr::Paren(paren) => self.rooted_expr_name(&paren.expr),
            _ => None,
        }
    }

    fn parameter_aliases(&self) -> BTreeMap<(usize, String), BindingProvenance> {
        let mut aliases = BTreeMap::<(usize, String), Option<String>>::new();
        for (callee, arguments) in &self.calls {
            let Some((scope, parameters)) = self.functions.get(callee) else {
                continue;
            };
            for (parameter, target) in parameters.iter().zip(arguments) {
                let entry = aliases
                    .entry((*scope, parameter.clone()))
                    .or_insert_with(|| target.clone());
                if entry != target {
                    *entry = None;
                }
            }
        }
        aliases
            .into_iter()
            .filter_map(|(key, target)| {
                target.map(|target| (key, BindingProvenance::ValueAlias { target }))
            })
            .collect()
    }
}

impl Visit for AliasCollector {
    fn visit_import_decl(&mut self, import: &ImportDecl) {
        let scope = self.current_scope();
        let module = import.src.value.to_string_lossy().to_string();
        for specifier in &import.specifiers {
            match specifier {
                ImportSpecifier::Named(named) => {
                    let local = named.local.sym.to_string();
                    let export = named
                        .imported
                        .as_ref()
                        .map(module_export_name)
                        .unwrap_or_else(|| local.clone());
                    self.insert(
                        scope,
                        local,
                        BindingProvenance::ModuleExport {
                            module: module.clone(),
                            export,
                        },
                    );
                }
                ImportSpecifier::Namespace(namespace) => self.insert(
                    scope,
                    namespace.local.sym.to_string(),
                    BindingProvenance::ModuleNamespace {
                        module: module.clone(),
                    },
                ),
                ImportSpecifier::Default(default) => {
                    self.insert_local(scope, default.local.sym.to_string());
                }
            }
        }
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        let scope = self.binding_scope(var_decl.kind);
        for declarator in &var_decl.decls {
            let value_alias = declarator
                .init
                .as_deref()
                .and_then(|init| self.rooted_expr_name(init));
            self.insert_pat_locals(scope, &declarator.name);
            if let Some(init) = declarator.init.as_deref()
                && let Some(module) = require_module_name(init)
            {
                collect_require_aliases(&declarator.name, module, scope, self);
            } else if let Some(target) = value_alias {
                collect_value_aliases(&declarator.name, &target, scope, self);
            }
        }
        var_decl.visit_children_with(self);
    }

    fn visit_assign_expr(&mut self, assignment: &AssignExpr) {
        let provenance = self
            .rooted_expr_name(&assignment.right)
            .map(|target| BindingProvenance::ValueAlias { target })
            .unwrap_or(BindingProvenance::Local);
        match &assignment.left {
            AssignTarget::Simple(SimpleAssignTarget::Ident(ident)) => {
                if let Some((scope, _)) = self.stack.iter().rev().find_map(|scope| {
                    self.scopes[*scope]
                        .bindings
                        .contains_key(ident.id.sym.as_ref())
                        .then_some((*scope, ()))
                }) {
                    self.record_assignment(
                        assignment.span,
                        scope,
                        ident.id.sym.to_string(),
                        provenance,
                    );
                }
            }
            AssignTarget::Simple(SimpleAssignTarget::Member(member)) => {
                if let Some(property) = member_chain(member) {
                    self.property_assignments.push(PropertyAliasAssignment {
                        span: assignment.span,
                        scope: self.current_scope(),
                        property,
                        target: self.rooted_expr_name(&assignment.right),
                    });
                }
            }
            AssignTarget::Pat(pattern) => {
                let pattern: Pat = pattern.clone().into();
                if let Some(target) = self.rooted_expr_name(&assignment.right) {
                    collect_assignment_aliases(
                        &pattern,
                        &target,
                        assignment.span,
                        self.current_scope(),
                        self,
                    );
                }
            }
            _ => {}
        }
        assignment.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, call: &CallExpr) {
        if let Callee::Expr(callee) = &call.callee
            && let Expr::Ident(callee) = &**callee
        {
            self.calls.push((
                callee.sym.to_string(),
                call.args
                    .iter()
                    .map(|argument| self.rooted_expr_name(&argument.expr))
                    .collect(),
            ));
        }
        call.visit_children_with(self);
    }

    fn visit_fn_decl(&mut self, fn_decl: &FnDecl) {
        let parent = self.current_scope();
        self.insert_local(parent, fn_decl.ident.sym.to_string());
        self.push_scope(fn_decl.function.span, ScopeKind::Function);
        let scope = self.current_scope();
        let parameters = fn_decl
            .function
            .params
            .iter()
            .filter_map(|parameter| binding_ident_name(&parameter.pat))
            .collect::<Vec<_>>();
        for parameter in &fn_decl.function.params {
            self.insert_pat_locals(scope, &parameter.pat);
        }
        self.functions
            .insert(fn_decl.ident.sym.to_string(), (scope, parameters));
        fn_decl.function.decorators.visit_with(self);
        fn_decl.function.body.visit_with(self);
        self.pop_scope();
    }

    fn visit_class_decl(&mut self, class_decl: &ClassDecl) {
        let scope = self.current_scope();
        self.insert_local(scope, class_decl.ident.sym.to_string());
        class_decl.class.visit_children_with(self);
    }

    fn visit_function(&mut self, function: &Function) {
        self.push_scope(function.span, ScopeKind::Function);
        let scope = self.current_scope();
        for param in &function.params {
            self.insert_pat_locals(scope, &param.pat);
        }
        function.decorators.visit_with(self);
        function.body.visit_with(self);
        self.pop_scope();
    }

    fn visit_arrow_expr(&mut self, arrow: &ArrowExpr) {
        self.push_scope(arrow.span, ScopeKind::Function);
        let scope = self.current_scope();
        for param in &arrow.params {
            self.insert_pat_locals(scope, param);
        }
        arrow.body.visit_with(self);
        self.pop_scope();
    }

    fn visit_block_stmt(&mut self, block: &BlockStmt) {
        self.push_scope(block.span, ScopeKind::Block);
        block.stmts.visit_with(self);
        self.pop_scope();
    }

    fn visit_catch_clause(&mut self, catch: &CatchClause) {
        self.push_scope(catch.span, ScopeKind::Block);
        let scope = self.current_scope();
        if let Some(param) = &catch.param {
            self.insert_pat_locals(scope, param);
        }
        catch.body.stmts.visit_with(self);
        self.pop_scope();
    }
}

fn collect_value_aliases(pat: &Pat, target: &str, scope: usize, aliases: &mut AliasCollector) {
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

fn collect_assignment_aliases(
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

fn contains(outer: Span, inner: Span) -> bool {
    outer.lo <= inner.lo && outer.hi >= inner.hi
}

fn member_prefix_ends(chain: &str) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(chain.len()).chain(chain.rmatch_indices('.').map(|(index, _)| index))
}

fn collect_require_aliases(pat: &Pat, module: String, scope: usize, aliases: &mut AliasCollector) {
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

fn member_root_ident(member: &MemberExpr) -> Option<&Ident> {
    expr_root_ident(&member.obj)
}

fn expr_root_ident(expr: &Expr) -> Option<&Ident> {
    match expr {
        Expr::Ident(ident) => Some(ident),
        Expr::Member(parent) => member_root_ident(parent),
        Expr::OptChain(chain) => match &*chain.base {
            OptChainBase::Member(member) => member_root_ident(member),
            OptChainBase::Call(call) => expr_root_ident(&call.callee),
        },
        Expr::Paren(paren) => expr_root_ident(&paren.expr),
        _ => None,
    }
}

fn expr_member(expr: &Expr) -> Option<&MemberExpr> {
    match expr {
        Expr::Member(member) => Some(member),
        Expr::OptChain(chain) => match &*chain.base {
            OptChainBase::Member(member) => Some(member),
            OptChainBase::Call(call) => expr_member(&call.callee),
        },
        Expr::Paren(paren) => expr_member(&paren.expr),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SymbolCallProvenance {
    Global,
    Local,
    ModuleExport { module: String, export: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SymbolMemberProvenance {
    ModuleNamespace { module: String, member: String },
}

fn collect_pat_bindings(pat: &Pat, bindings: &mut BTreeSet<String>) {
    match pat {
        Pat::Ident(ident) => {
            bindings.insert(ident.id.sym.to_string());
        }
        Pat::Array(array) => {
            for elem in array.elems.iter().flatten() {
                collect_pat_bindings(elem, bindings);
            }
        }
        Pat::Rest(rest) => collect_pat_bindings(&rest.arg, bindings),
        Pat::Object(object) => {
            for prop in &object.props {
                match prop {
                    ObjectPatProp::KeyValue(key_value) => {
                        collect_pat_bindings(&key_value.value, bindings);
                    }
                    ObjectPatProp::Assign(assign) => {
                        bindings.insert(assign.key.sym.to_string());
                    }
                    ObjectPatProp::Rest(rest) => collect_pat_bindings(&rest.arg, bindings),
                }
            }
        }
        Pat::Assign(assign) => collect_pat_bindings(&assign.left, bindings),
        Pat::Invalid(_) | Pat::Expr(_) => {}
    }
}

fn binding_ident_name(pat: &Pat) -> Option<String> {
    match pat {
        Pat::Ident(ident) => Some(ident.id.sym.to_string()),
        Pat::Assign(assign) => binding_ident_name(&assign.left),
        _ => None,
    }
}

pub(super) fn require_module_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Call(call) => require_call_module_name(call)
            .or_else(|| {
                call.args
                    .iter()
                    .find_map(|arg| require_module_name(&arg.expr))
            })
            .or_else(|| match &call.callee {
                Callee::Expr(callee) => require_module_name(callee),
                Callee::Super(_) | Callee::Import(_) => None,
            }),
        Expr::Member(member) => require_module_name(&member.obj),
        Expr::Paren(paren) => require_module_name(&paren.expr),
        _ => None,
    }
}

fn require_call_module_name(call: &CallExpr) -> Option<String> {
    let Callee::Expr(callee) = &call.callee else {
        return None;
    };
    let Expr::Ident(ident) = &**callee else {
        return None;
    };
    if ident.sym != *"require" {
        return None;
    }

    call.args.first().and_then(|arg| match &*arg.expr {
        Expr::Lit(Lit::Str(value)) => Some(value.value.to_string_lossy().to_string()),
        _ => None,
    })
}

fn module_export_name(name: &ModuleExportName) -> String {
    match name {
        ModuleExportName::Ident(ident) => ident.sym.to_string(),
        ModuleExportName::Str(value) => value.value.to_string_lossy().to_string(),
    }
}

fn prop_name(name: &swc_ecma_ast::PropName) -> Option<String> {
    match name {
        swc_ecma_ast::PropName::Ident(ident) => Some(ident.sym.to_string()),
        swc_ecma_ast::PropName::Str(value) => Some(value.value.to_string_lossy().to_string()),
        swc_ecma_ast::PropName::Num(number) => Some(number.value.to_string()),
        swc_ecma_ast::PropName::Computed(computed) => {
            if let Expr::Lit(Lit::Str(value)) = &*computed.expr {
                Some(value.value.to_string_lossy().to_string())
            } else {
                None
            }
        }
        swc_ecma_ast::PropName::BigInt(_) => None,
    }
}

pub(super) fn expr_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Ident(ident) => Some(ident.sym.to_string()),
        Expr::Member(member) => member_chain(member),
        Expr::This(_) => Some("this".to_string()),
        Expr::OptChain(chain) => match &*chain.base {
            OptChainBase::Member(member) => member_chain(member),
            OptChainBase::Call(call) => expr_name(&call.callee),
        },
        Expr::Paren(paren) => expr_name(&paren.expr),
        Expr::TsAs(expr) => expr_name(&expr.expr),
        Expr::TsNonNull(expr) => expr_name(&expr.expr),
        Expr::TsSatisfies(expr) => expr_name(&expr.expr),
        Expr::TsTypeAssertion(expr) => expr_name(&expr.expr),
        _ => None,
    }
}

pub(super) fn member_chain(member: &MemberExpr) -> Option<String> {
    let object = expr_name(&member.obj)?;
    let prop = member_prop_name(&member.prop)?;
    Some(format!("{object}.{prop}"))
}

pub(super) fn member_prop_name(prop: &MemberProp) -> Option<String> {
    match prop {
        MemberProp::Ident(ident) => Some(ident.sym.to_string()),
        MemberProp::PrivateName(name) => Some(format!("#{}", name.name)),
        MemberProp::Computed(computed) => static_property_name(&computed.expr),
    }
}

fn static_property_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(Lit::Str(value)) => Some(value.value.to_string_lossy().to_string()),
        Expr::Lit(Lit::Num(value)) => Some(value.value.to_string()),
        Expr::Tpl(template) if template.exprs.is_empty() && template.quasis.len() == 1 => {
            template.quasis.first().map(|quasi| quasi.raw.to_string())
        }
        Expr::Bin(binary) if binary.op == BinaryOp::Add => Some(format!(
            "{}{}",
            static_property_name(&binary.left)?,
            static_property_name(&binary.right)?
        )),
        Expr::Paren(paren) => static_property_name(&paren.expr),
        _ => None,
    }
}

fn literal_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(Lit::Str(value)) => Some(value.value.to_string_lossy().to_string()),
        Expr::Tpl(template) if template.exprs.is_empty() && template.quasis.len() == 1 => {
            template.quasis.first().map(|quasi| quasi.raw.to_string())
        }
        Expr::Paren(paren) => literal_string(&paren.expr),
        _ => None,
    }
}
