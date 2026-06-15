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
    Bundle,
    Dependency,
    DynamicCode,
}

impl ApiCategory {
    pub(in crate::plugins::analysis) fn as_str(self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::Vault => "vault",
            Self::Metadata => "metadata",
            Self::Workspace => "workspace",
            Self::Editor => "editor",
            Self::Ui => "ui",
            Self::Settings => "settings",
            Self::Lifecycle => "lifecycle",
            Self::Filesystem => "filesystem",
            Self::Electron => "electron",
            Self::Browser => "browser",
            Self::Bundle => "bundle",
            Self::Dependency => "dependency",
            Self::DynamicCode => "dynamic_code",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::plugins::analysis) enum ApiSeverity {
    Info,
    Warning,
}

impl ApiSeverity {
    pub(in crate::plugins::analysis) fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::plugins::analysis) enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub(in crate::plugins::analysis) fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}
