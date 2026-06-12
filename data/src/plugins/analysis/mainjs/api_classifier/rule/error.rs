#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(in crate::plugins::analysis::mainjs::api_classifier) enum ApiRuleBuildError {
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
}
