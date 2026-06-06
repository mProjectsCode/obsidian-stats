use std::collections::{BTreeMap, BTreeSet};

use swc_ecma_ast::{
    ArrowExpr, CallExpr, Callee, Expr, FnDecl, Function, Ident, ImportDecl, ImportSpecifier, Lit,
    MemberExpr, MemberProp, ModuleExportName, NewExpr, ObjectPatProp, Pat, Program, Str, VarDecl,
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
    module_member_calls: BTreeMap<(String, String), u32>,
    member_reads: BTreeMap<String, u32>,
    imports: BTreeMap<String, u32>,
    string_literals: BTreeMap<String, u32>,
    classes: BTreeMap<String, u32>,
    constructors: BTreeMap<String, u32>,
}

impl SymbolIndex {
    pub(super) fn collect(source: &str, program: Option<&Program>, rules: &[ApiRule]) -> Self {
        let mut index = Self::default();
        if let Some(program) = program {
            let aliases = AliasInfo::collect(program);

            let mut visitor = SymbolIndexVisitor {
                index: &mut index,
                aliases,
                callee_member_reads_to_skip: BTreeMap::new(),
            };
            program.visit_with(&mut visitor);
            index.collect_string_marker_fallback(source, rules);
        } else {
            index.collect_source_fallback(source, rules);
        }

        index
    }

    fn collect_source_fallback(&mut self, source: &str, rules: &[ApiRule]) {
        for rule in rules {
            for call in &rule.matcher.calls {
                match &call.provenance {
                    CallProvenance::Any => {
                        if source.contains(&call.name) {
                            self.increment(ApiMatchKind::Call, call.name.clone());
                        }
                    }
                    CallProvenance::Global => {
                        if source.contains(&call.name) {
                            self.increment(ApiMatchKind::Call, call.name.clone());
                            *self.global_calls.entry(call.name.clone()).or_insert(0) += 1;
                        }
                    }
                    CallProvenance::ModuleExport { .. } => {}
                }
            }

            for call in &rule.matcher.member_calls {
                if matches!(call.provenance, MemberCallProvenance::Any)
                    && source.contains(&call.chain)
                {
                    self.increment(ApiMatchKind::MemberCall, call.chain.clone());
                }
            }

            for constructor in &rule.matcher.constructors {
                if source.contains(constructor) {
                    self.increment(ApiMatchKind::Constructor, constructor.clone());
                }
            }

            for class in &rule.matcher.classes {
                if source.contains(class) {
                    self.increment(ApiMatchKind::Class, class.clone());
                }
            }

            for marker in &rule.matcher.string_literals {
                self.increment_source_string_marker(source, marker);
            }
        }
    }

    fn collect_string_marker_fallback(&mut self, source: &str, rules: &[ApiRule]) {
        for rule in rules {
            for marker in &rule.matcher.string_literals {
                self.increment_source_string_marker(source, marker);
            }
        }
    }

    fn increment_source_string_marker(&mut self, source: &str, marker: &str) {
        let count = source.matches(marker).count();
        if count > 0 {
            let count = saturating_u32(count);
            self.string_literals
                .entry(marker.to_string())
                .and_modify(|existing| *existing = (*existing).max(count))
                .or_insert(count);
        }
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
}

struct SymbolIndexVisitor<'a> {
    index: &'a mut SymbolIndex,
    aliases: AliasInfo,
    callee_member_reads_to_skip: BTreeMap<String, u32>,
}

impl Visit for SymbolIndexVisitor<'_> {
    fn visit_import_decl(&mut self, import: &ImportDecl) {
        self.index
            .increment(ApiMatchKind::Import, import.src.value.to_string_lossy());

        import.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, call: &CallExpr) {
        match &call.callee {
            Callee::Expr(callee) => match &**callee {
                Expr::Ident(ident) => {
                    let name = ident.sym.to_string();
                    self.index.increment(ApiMatchKind::Call, name.clone());
                    match self.aliases.call_provenance(&name) {
                        SymbolCallProvenance::Global => {
                            *self.index.global_calls.entry(name).or_insert(0) += 1;
                        }
                        SymbolCallProvenance::ModuleExport { module, export } => {
                            *self.index.module_calls.entry((module, export)).or_insert(0) += 1;
                        }
                        SymbolCallProvenance::Local => {}
                    }
                }
                Expr::Member(member) => {
                    if let Some(chain) = member_chain(member) {
                        self.index
                            .increment(ApiMatchKind::MemberCall, chain.clone());
                        if let Some(SymbolMemberProvenance::ModuleNamespace { module, member }) =
                            self.aliases.member_call_provenance(member)
                        {
                            *self
                                .index
                                .module_member_calls
                                .entry((module, member))
                                .or_insert(0) += 1;
                        }
                        *self.callee_member_reads_to_skip.entry(chain).or_insert(0) += 1;
                    }
                }
                _ => {}
            },
            Callee::Super(_) => self.index.increment(ApiMatchKind::Call, "super"),
            Callee::Import(_) => self.index.increment(ApiMatchKind::Call, "import"),
        }

        call.visit_children_with(self);
    }

    fn visit_member_expr(&mut self, member: &MemberExpr) {
        if let Some(chain) = member_chain(member) {
            if let Some(skip_count) = self.callee_member_reads_to_skip.get_mut(&chain) {
                *skip_count -= 1;
                if *skip_count == 0 {
                    self.callee_member_reads_to_skip.remove(&chain);
                }

                member.visit_children_with(self);
                return;
            }

            self.index.increment(ApiMatchKind::MemberRead, chain);
        }

        member.visit_children_with(self);
    }

    fn visit_new_expr(&mut self, new_expr: &NewExpr) {
        if let Some(name) = expr_name(&new_expr.callee) {
            self.index
                .increment(ApiMatchKind::Constructor, name.clone());
            self.index.increment(ApiMatchKind::Class, name);
        }

        new_expr.visit_children_with(self);
    }

    fn visit_ident(&mut self, ident: &Ident) {
        self.index
            .increment(ApiMatchKind::Class, ident.sym.to_string());
    }

    fn visit_str(&mut self, value: &Str) {
        self.index
            .increment(ApiMatchKind::StringLiteral, value.value.to_string_lossy());

        value.visit_children_with(self);
    }
}

#[derive(Debug, Default, Clone)]
pub(super) struct AliasInfo {
    local_bindings: BTreeSet<String>,
    module_export_by_local: BTreeMap<String, (String, String)>,
    module_by_namespace_local: BTreeMap<String, String>,
}

impl AliasInfo {
    pub(super) fn collect(program: &Program) -> Self {
        let mut aliases = Self::default();
        program.visit_with(&mut aliases);
        aliases
    }

    pub(super) fn call_provenance(&self, name: &str) -> SymbolCallProvenance {
        if let Some((module, export)) = self.module_export_by_local.get(name) {
            SymbolCallProvenance::ModuleExport {
                module: module.clone(),
                export: export.clone(),
            }
        } else if self.local_bindings.contains(name) {
            SymbolCallProvenance::Local
        } else {
            SymbolCallProvenance::Global
        }
    }

    pub(super) fn member_call_provenance(
        &self,
        member: &MemberExpr,
    ) -> Option<SymbolMemberProvenance> {
        let member_name = member_prop_name(&member.prop)?;
        let object = expr_name(&member.obj)?;
        self.module_by_namespace_local.get(&object).map(|module| {
            SymbolMemberProvenance::ModuleNamespace {
                module: module.clone(),
                member: member_name,
            }
        })
    }

    fn insert_binding(&mut self, name: impl Into<String>) {
        self.local_bindings.insert(name.into());
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

impl Visit for AliasInfo {
    fn visit_import_decl(&mut self, import: &ImportDecl) {
        let module = import.src.value.to_string_lossy().to_string();
        for specifier in &import.specifiers {
            match specifier {
                ImportSpecifier::Named(named) => {
                    let local = named.local.sym.to_string();
                    let imported = named
                        .imported
                        .as_ref()
                        .map(module_export_name)
                        .unwrap_or_else(|| local.clone());
                    self.module_export_by_local
                        .insert(local.clone(), (module.clone(), imported));
                    self.insert_binding(local);
                }
                ImportSpecifier::Namespace(namespace) => {
                    let local = namespace.local.sym.to_string();
                    self.module_by_namespace_local
                        .insert(local.clone(), module.clone());
                    self.insert_binding(local);
                }
                ImportSpecifier::Default(default) => {
                    self.insert_binding(default.local.sym.to_string());
                }
            }
        }

        import.visit_children_with(self);
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        for declarator in &var_decl.decls {
            collect_pat_bindings(&declarator.name, &mut self.local_bindings);

            if let Some(init) = declarator.init.as_deref()
                && let Some(module) = require_module_name(init)
            {
                collect_require_aliases(&declarator.name, module, self);
            }
        }

        var_decl.visit_children_with(self);
    }

    fn visit_fn_decl(&mut self, fn_decl: &FnDecl) {
        self.insert_binding(fn_decl.ident.sym.to_string());
        fn_decl.visit_children_with(self);
    }

    fn visit_function(&mut self, function: &Function) {
        for param in &function.params {
            collect_pat_bindings(&param.pat, &mut self.local_bindings);
        }

        function.visit_children_with(self);
    }

    fn visit_arrow_expr(&mut self, arrow: &ArrowExpr) {
        for param in &arrow.params {
            collect_pat_bindings(param, &mut self.local_bindings);
        }

        arrow.visit_children_with(self);
    }
}

fn collect_require_aliases(pat: &Pat, module: String, aliases: &mut AliasInfo) {
    match pat {
        Pat::Ident(ident) => {
            aliases
                .module_by_namespace_local
                .insert(ident.id.sym.to_string(), module);
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
                                aliases,
                            );
                        }
                    }
                    ObjectPatProp::Assign(assign) => {
                        let local = assign.key.sym.to_string();
                        aliases
                            .module_export_by_local
                            .insert(local.clone(), (module.clone(), local));
                    }
                    ObjectPatProp::Rest(_) => {}
                }
            }
        }
        _ => {}
    }
}

fn collect_require_export_alias(pat: &Pat, module: &str, export: &str, aliases: &mut AliasInfo) {
    if let Pat::Ident(local) = pat {
        aliases.module_export_by_local.insert(
            local.id.sym.to_string(),
            (module.to_string(), export.to_string()),
        );
    }
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
        MemberProp::Computed(computed) => {
            if let Expr::Lit(Lit::Str(value)) = &*computed.expr {
                Some(value.value.to_string_lossy().to_string())
            } else {
                None
            }
        }
    }
}

fn saturating_u32(value: usize) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}
