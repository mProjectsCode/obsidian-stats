use super::rule::{ApiCategory, ApiSeverity, Confidence};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::plugins::analysis) struct ApiCapability {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) category: ApiCategory,
    pub(super) severity: ApiSeverity,
    pub(super) confidence: Confidence,
    pub(super) evidence: Vec<ApiEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::plugins::analysis) struct Disclosure {
    pub(super) id: String,
    pub(super) from_capability: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::plugins::analysis) struct ApiEvidence {
    pub(super) kind: ApiMatchKind,
    pub(super) symbol: String,
    pub(super) count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::plugins::analysis) enum ApiMatchKind {
    Call,
    MemberCall,
    MemberRead,
    Import,
    StringLiteral,
    Class,
    Constructor,
    CallArgument,
    CustomAst,
    Correlation,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(in crate::plugins::analysis) struct ApiClassificationResult {
    pub(super) capabilities: Vec<ApiCapability>,
    pub(super) disclosures: Vec<Disclosure>,
}

impl ApiClassificationResult {
    pub(in crate::plugins::analysis) fn capabilities(&self) -> &[ApiCapability] {
        &self.capabilities
    }

    pub(in crate::plugins::analysis) fn disclosures(&self) -> &[Disclosure] {
        &self.disclosures
    }

    pub(in crate::plugins::analysis::mainjs) fn has_capability(&self, id: &str) -> bool {
        self.capabilities
            .iter()
            .any(|capability| capability.id == id)
    }

    pub(in crate::plugins::analysis::mainjs) fn has_disclosure(&self, id: &str) -> bool {
        self.disclosures
            .iter()
            .any(|disclosure| disclosure.id == id)
    }
}

impl ApiMatchKind {
    pub(in crate::plugins::analysis) fn as_str(self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::MemberCall => "member_call",
            Self::MemberRead => "member_read",
            Self::Import => "import",
            Self::StringLiteral => "string_literal",
            Self::Class => "class",
            Self::Constructor => "constructor",
            Self::CallArgument => "call_argument",
            Self::CustomAst => "custom_ast",
            Self::Correlation => "correlation",
        }
    }
}

impl ApiCapability {
    pub(in crate::plugins::analysis) fn id(&self) -> &str {
        &self.id
    }

    pub(in crate::plugins::analysis) fn label(&self) -> &str {
        &self.label
    }

    pub(in crate::plugins::analysis) fn category(&self) -> ApiCategory {
        self.category
    }

    pub(in crate::plugins::analysis) fn severity(&self) -> ApiSeverity {
        self.severity
    }

    pub(in crate::plugins::analysis) fn confidence(&self) -> Confidence {
        self.confidence
    }

    pub(in crate::plugins::analysis) fn evidence(&self) -> &[ApiEvidence] {
        &self.evidence
    }
}

impl Disclosure {
    pub(in crate::plugins::analysis) fn id(&self) -> &str {
        &self.id
    }

    pub(in crate::plugins::analysis) fn source_capability(&self) -> &str {
        &self.from_capability
    }
}

impl ApiEvidence {
    pub(in crate::plugins::analysis) fn kind(&self) -> ApiMatchKind {
        self.kind
    }

    pub(in crate::plugins::analysis) fn symbol(&self) -> &str {
        &self.symbol
    }

    pub(in crate::plugins::analysis) fn count(&self) -> u32 {
        self.count
    }
}
