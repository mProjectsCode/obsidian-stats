use super::context::ApiMatchContext;
use super::result::ApiEvidence;

pub(super) type CustomAstMatcher = fn(&ApiMatchContext<'_>) -> Vec<ApiEvidence>;

#[derive(Debug, Clone)]
pub(in crate::plugins::analysis::mainjs) struct ApiRule {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) category: ApiCategory,
    pub(super) severity: ApiSeverity,
    pub(super) confidence: Confidence,
    pub(super) matcher: ApiMatcher,
    pub(super) implies: Vec<String>,
    pub(super) when_all: Vec<String>,
    pub(super) when_any: Vec<String>,
    pub(super) evidence_limit: usize,
    pub(super) min_distinct_evidence: usize,
}

impl ApiRule {
    const DEFAULT_EVIDENCE_LIMIT: usize = 5;

    pub(super) fn builder(id: impl Into<String>) -> ApiRuleBuilder {
        ApiRuleBuilder {
            id: id.into(),
            label: None,
            category: None,
            severity: None,
            confidence: None,
            matcher: ApiMatcher::default(),
            implies: Vec::new(),
            when_all: Vec::new(),
            when_any: Vec::new(),
            evidence_limit: Self::DEFAULT_EVIDENCE_LIMIT,
            min_distinct_evidence: 1,
        }
    }

    pub(super) fn is_correlation(&self) -> bool {
        !self.when_all.is_empty() || !self.when_any.is_empty()
    }
}

#[derive(Debug, Clone)]
pub(super) struct ApiRuleBuilder {
    id: String,
    label: Option<String>,
    category: Option<ApiCategory>,
    severity: Option<ApiSeverity>,
    confidence: Option<Confidence>,
    matcher: ApiMatcher,
    implies: Vec<String>,
    when_all: Vec<String>,
    when_any: Vec<String>,
    evidence_limit: usize,
    min_distinct_evidence: usize,
}

impl ApiRuleBuilder {
    pub(super) fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub(super) fn category(mut self, category: ApiCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub(super) fn severity(mut self, severity: ApiSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    pub(super) fn confidence(mut self, confidence: Confidence) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub(super) fn calls<I, S>(mut self, calls: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.matcher.calls.extend(
            calls
                .into_iter()
                .map(Into::into)
                .map(CallMatcher::unqualified),
        );
        self
    }

    pub(super) fn global_calls<I, S>(mut self, calls: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.matcher
            .calls
            .extend(calls.into_iter().map(Into::into).map(CallMatcher::global));
        self
    }

    pub(super) fn module_calls<I, S>(mut self, module: impl Into<String>, exports: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let module = module.into();
        self.matcher.calls.extend(
            exports
                .into_iter()
                .map(Into::into)
                .map(|name| CallMatcher::module_export(module.clone(), name)),
        );
        self
    }

    pub(super) fn member_calls<I, S>(mut self, member_calls: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.matcher.member_calls.extend(
            member_calls
                .into_iter()
                .map(Into::into)
                .map(MemberCallMatcher::chain),
        );
        self
    }

    pub(super) fn member_call(mut self, member_call: impl Into<String>) -> Self {
        self.matcher
            .member_calls
            .push(MemberCallMatcher::chain(member_call.into()));
        self
    }

    pub(super) fn arg_string<I, S>(mut self, index: usize, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        if let Some(call) = self.matcher.member_calls.last_mut() {
            call.arg_strings.push(ArgStringMatcher {
                index,
                values: values.into_iter().map(Into::into).collect(),
            });
        }
        self
    }

    pub(super) fn module_member_calls<I, S>(mut self, module: impl Into<String>, members: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let module = module.into();
        self.matcher.member_calls.extend(
            members
                .into_iter()
                .map(Into::into)
                .map(|member| MemberCallMatcher::module_member(module.clone(), member)),
        );
        self
    }

    pub(super) fn member_reads<I, S>(mut self, member_reads: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.matcher
            .member_reads
            .extend(member_reads.into_iter().map(Into::into));
        self
    }

    pub(super) fn imports<I, S>(mut self, imports: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.matcher
            .imports
            .extend(imports.into_iter().map(Into::into));
        self
    }

    pub(super) fn string_literals<I, S>(mut self, string_literals: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.matcher
            .string_literals
            .extend(string_literals.into_iter().map(Into::into));
        self
    }

    pub(super) fn classes<I, S>(mut self, classes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.matcher
            .classes
            .extend(classes.into_iter().map(Into::into));
        self
    }

    pub(super) fn constructors<I, S>(mut self, constructors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.matcher
            .constructors
            .extend(constructors.into_iter().map(Into::into));
        self
    }

    pub(super) fn custom_ast(mut self, name: impl Into<String>, matcher: CustomAstMatcher) -> Self {
        self.matcher.custom_ast.push(CustomMatcher {
            name: name.into(),
            matcher,
        });
        self
    }

    pub(super) fn implies<I, S>(mut self, implies: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.implies.extend(implies.into_iter().map(Into::into));
        self
    }

    pub(super) fn when_all<I, S>(mut self, when_all: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.when_all.extend(when_all.into_iter().map(Into::into));
        self
    }

    pub(super) fn requires_all<I, S>(self, dependencies: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.when_all(dependencies)
    }

    pub(super) fn when_any<I, S>(mut self, when_any: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.when_any.extend(when_any.into_iter().map(Into::into));
        self
    }

    pub(super) fn requires_any<I, S>(self, dependencies: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.when_any(dependencies)
    }

    pub(super) fn evidence_limit(mut self, evidence_limit: usize) -> Self {
        self.evidence_limit = evidence_limit;
        self
    }

    #[allow(dead_code)]
    pub(super) fn min_distinct_evidence(mut self, min_distinct_evidence: usize) -> Self {
        self.min_distinct_evidence = min_distinct_evidence.max(1);
        self
    }

    pub(super) fn build(self) -> Result<ApiRule, ApiRuleBuildError> {
        let label = required_string(self.label, ApiRuleBuildError::MissingLabel)?;
        let category = self.category.ok_or(ApiRuleBuildError::MissingCategory)?;
        let severity = self.severity.ok_or(ApiRuleBuildError::MissingSeverity)?;
        let confidence = self
            .confidence
            .ok_or(ApiRuleBuildError::MissingConfidence)?;

        let id = self.id.trim().to_string();
        if id.is_empty() {
            return Err(ApiRuleBuildError::MissingId);
        }

        let matcher = self.matcher.normalized();
        let implies = normalized_strings(self.implies);
        let when_all = normalized_strings(self.when_all);
        let when_any = normalized_strings(self.when_any);

        let has_primitive_matcher = !matcher.is_empty();
        let has_correlation = !when_all.is_empty() || !when_any.is_empty();
        if !has_primitive_matcher && !has_correlation {
            return Err(ApiRuleBuildError::MissingMatcher);
        }
        Ok(ApiRule {
            id,
            label,
            category,
            severity,
            confidence,
            matcher,
            implies,
            when_all,
            when_any,
            evidence_limit: self.evidence_limit,
            min_distinct_evidence: self.min_distinct_evidence,
        })
    }
}

fn required_string(
    value: Option<String>,
    missing_error: ApiRuleBuildError,
) -> Result<String, ApiRuleBuildError> {
    let value = value.ok_or(missing_error)?;
    if value.trim().is_empty() {
        return Err(missing_error);
    }

    Ok(value.trim().to_string())
}

fn normalized_strings(values: Vec<String>) -> Vec<String> {
    let mut values = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::plugins::analysis) enum ApiCategory {
    Network,
    Vault,
    Metadata,
    Workspace,
    Editor,
    Ui,
    Settings,
    Lifecycle,
    Filesystem,
    Electron,
    Browser,
    Dependency,
    DynamicCode,
    Correlation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::plugins::analysis) enum ApiSeverity {
    Info,
    Notice,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::plugins::analysis) enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Default)]
pub(super) struct ApiMatcher {
    pub(super) calls: Vec<CallMatcher>,
    pub(super) member_calls: Vec<MemberCallMatcher>,
    pub(super) member_reads: Vec<String>,
    pub(super) imports: Vec<String>,
    pub(super) string_literals: Vec<String>,
    pub(super) classes: Vec<String>,
    pub(super) constructors: Vec<String>,
    pub(super) custom_ast: Vec<CustomMatcher>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ArgStringMatcher {
    pub(super) index: usize,
    pub(super) values: Vec<String>,
}

#[derive(Debug, Clone)]
pub(super) struct CustomMatcher {
    pub(super) name: String,
    pub(super) matcher: CustomAstMatcher,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CallMatcher {
    pub(super) name: String,
    pub(super) provenance: CallProvenance,
}

impl CallMatcher {
    fn unqualified(name: String) -> Self {
        Self {
            name,
            provenance: CallProvenance::Any,
        }
    }

    fn global(name: String) -> Self {
        Self {
            name,
            provenance: CallProvenance::Global,
        }
    }

    fn module_export(module: String, export: String) -> Self {
        Self {
            name: export,
            provenance: CallProvenance::ModuleExport { module },
        }
    }

    pub(super) fn evidence_symbol(&self) -> String {
        match &self.provenance {
            CallProvenance::Any | CallProvenance::Global => self.name.clone(),
            CallProvenance::ModuleExport { module } => format!("{module}.{}", self.name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CallProvenance {
    Any,
    Global,
    ModuleExport { module: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MemberCallMatcher {
    pub(super) chain: String,
    pub(super) provenance: MemberCallProvenance,
    pub(super) arg_strings: Vec<ArgStringMatcher>,
}

impl MemberCallMatcher {
    fn chain(chain: String) -> Self {
        Self {
            chain,
            provenance: MemberCallProvenance::Any,
            arg_strings: Vec::new(),
        }
    }

    fn module_member(module: String, member: String) -> Self {
        Self {
            chain: member,
            provenance: MemberCallProvenance::ModuleNamespace { module },
            arg_strings: Vec::new(),
        }
    }

    pub(super) fn evidence_symbol(&self) -> String {
        match &self.provenance {
            MemberCallProvenance::Any => self.chain.clone(),
            MemberCallProvenance::ModuleNamespace { module } => format!("{module}.{}", self.chain),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum MemberCallProvenance {
    Any,
    ModuleNamespace { module: String },
}

impl ApiMatcher {
    pub(super) fn is_empty(&self) -> bool {
        self.calls.is_empty()
            && self.member_calls.is_empty()
            && self.member_reads.is_empty()
            && self.imports.is_empty()
            && self.string_literals.is_empty()
            && self.classes.is_empty()
            && self.constructors.is_empty()
            && self.custom_ast.is_empty()
    }

    fn normalized(mut self) -> Self {
        self.calls
            .sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
        self.calls.dedup();
        self.member_calls
            .sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
        self.member_calls.dedup();
        normalize_strings(&mut self.member_reads);
        normalize_strings(&mut self.imports);
        normalize_strings(&mut self.string_literals);
        normalize_strings(&mut self.classes);
        normalize_strings(&mut self.constructors);
        for member_call in &mut self.member_calls {
            for matcher in &mut member_call.arg_strings {
                normalize_strings(&mut matcher.values);
            }
        }
        self.custom_ast
            .retain(|matcher| !matcher.name.trim().is_empty());
        for matcher in &mut self.custom_ast {
            matcher.name = matcher.name.trim().to_string();
        }
        self.custom_ast
            .sort_by(|left, right| left.name.cmp(&right.name));
        self.custom_ast
            .dedup_by(|left, right| left.name == right.name);
        self
    }
}

fn normalize_strings(values: &mut Vec<String>) {
    values.retain(|value| !value.trim().is_empty());
    for value in values.iter_mut() {
        *value = value.trim().to_string();
    }
    values.sort();
    values.dedup();
}

impl CallMatcher {
    fn sort_key(&self) -> (&str, &str) {
        match &self.provenance {
            CallProvenance::Any => ("any", &self.name),
            CallProvenance::Global => ("global", &self.name),
            CallProvenance::ModuleExport { module } => (module, &self.name),
        }
    }
}

impl MemberCallMatcher {
    fn sort_key(&self) -> (&str, &str) {
        match &self.provenance {
            MemberCallProvenance::Any => ("any", &self.chain),
            MemberCallProvenance::ModuleNamespace { module } => (module, &self.chain),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ApiRuleBuildError {
    MissingId,
    MissingLabel,
    MissingCategory,
    MissingSeverity,
    MissingConfidence,
    MissingMatcher,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::plugins::analysis::mainjs) enum ApiCatalogError {
    DuplicateRule(String),
    UnknownDisclosure(String),
    UnknownRule(String),
}
