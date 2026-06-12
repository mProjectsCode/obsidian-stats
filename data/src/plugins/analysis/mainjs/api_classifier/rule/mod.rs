use super::context::ApiMatchContext;
use super::result::ApiEvidence;

mod error;
mod matcher;
mod taxonomy;

pub(super) use error::{ApiCatalogError, ApiRuleBuildError};
pub(super) use matcher::{
    ApiMatcher, ArgStringMatcher, CallMatcher, CallProvenance, CustomMatcher, MemberCallMatcher,
    MemberCallProvenance, canonical_rooted_chain,
};
pub(in crate::plugins::analysis) use taxonomy::{ApiCategory, ApiSeverity, Confidence};

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
}

impl ApiRule {
    pub(super) const EVIDENCE_LIMIT: usize = 5;

    pub(super) fn builder(id: impl Into<String>) -> ApiRuleBuilder {
        ApiRuleBuilder {
            id: id.into(),
            label: None,
            category: None,
            severity: None,
            confidence: None,
            matcher: ApiMatcher::default(),
            implies: Vec::new(),
        }
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

    pub(super) fn rooted_member_calls<I, S>(mut self, member_calls: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.matcher.member_calls.extend(
            member_calls
                .into_iter()
                .map(Into::into)
                .map(MemberCallMatcher::rooted_chain),
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
        if matcher.is_empty() {
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
