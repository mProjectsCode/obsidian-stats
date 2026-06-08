use super::CustomAstMatcher;

#[derive(Debug, Clone, Default)]
pub(in crate::plugins::analysis::mainjs::api_classifier) struct ApiMatcher {
    pub(in crate::plugins::analysis::mainjs::api_classifier) calls: Vec<CallMatcher>,
    pub(in crate::plugins::analysis::mainjs::api_classifier) member_calls: Vec<MemberCallMatcher>,
    pub(in crate::plugins::analysis::mainjs::api_classifier) member_reads: Vec<String>,
    pub(in crate::plugins::analysis::mainjs::api_classifier) imports: Vec<String>,
    pub(in crate::plugins::analysis::mainjs::api_classifier) string_literals: Vec<String>,
    pub(in crate::plugins::analysis::mainjs::api_classifier) classes: Vec<String>,
    pub(in crate::plugins::analysis::mainjs::api_classifier) constructors: Vec<String>,
    pub(in crate::plugins::analysis::mainjs::api_classifier) custom_ast: Vec<CustomMatcher>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::plugins::analysis::mainjs::api_classifier) struct ArgStringMatcher {
    pub(in crate::plugins::analysis::mainjs::api_classifier) index: usize,
    pub(in crate::plugins::analysis::mainjs::api_classifier) values: Vec<String>,
}

#[derive(Debug, Clone)]
pub(in crate::plugins::analysis::mainjs::api_classifier) struct CustomMatcher {
    pub(in crate::plugins::analysis::mainjs::api_classifier) name: String,
    pub(in crate::plugins::analysis::mainjs::api_classifier) matcher: CustomAstMatcher,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::plugins::analysis::mainjs::api_classifier) struct CallMatcher {
    pub(in crate::plugins::analysis::mainjs::api_classifier) name: String,
    pub(in crate::plugins::analysis::mainjs::api_classifier) provenance: CallProvenance,
}

impl CallMatcher {
    pub(super) fn unqualified(name: String) -> Self {
        Self {
            name,
            provenance: CallProvenance::Any,
        }
    }

    pub(super) fn global(name: String) -> Self {
        Self {
            name,
            provenance: CallProvenance::Global,
        }
    }

    pub(super) fn module_export(module: String, export: String) -> Self {
        Self {
            name: export,
            provenance: CallProvenance::ModuleExport { module },
        }
    }

    pub(in crate::plugins::analysis::mainjs::api_classifier) fn evidence_symbol(&self) -> String {
        match &self.provenance {
            CallProvenance::Any | CallProvenance::Global => self.name.clone(),
            CallProvenance::ModuleExport { module } => format!("{module}.{}", self.name),
        }
    }

    fn sort_key(&self) -> (&str, &str) {
        match &self.provenance {
            CallProvenance::Any => ("any", &self.name),
            CallProvenance::Global => ("global", &self.name),
            CallProvenance::ModuleExport { module } => (module, &self.name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::plugins::analysis::mainjs::api_classifier) enum CallProvenance {
    Any,
    Global,
    ModuleExport { module: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::plugins::analysis::mainjs::api_classifier) struct MemberCallMatcher {
    pub(in crate::plugins::analysis::mainjs::api_classifier) chain: String,
    pub(in crate::plugins::analysis::mainjs::api_classifier) provenance: MemberCallProvenance,
    pub(in crate::plugins::analysis::mainjs::api_classifier) arg_strings: Vec<ArgStringMatcher>,
}

impl MemberCallMatcher {
    pub(super) fn chain(chain: String) -> Self {
        Self {
            chain,
            provenance: MemberCallProvenance::Any,
            arg_strings: Vec::new(),
        }
    }

    pub(super) fn rooted_chain(chain: String) -> Self {
        Self {
            chain,
            provenance: MemberCallProvenance::Rooted,
            arg_strings: Vec::new(),
        }
    }

    pub(super) fn module_member(module: String, member: String) -> Self {
        Self {
            chain: member,
            provenance: MemberCallProvenance::ModuleNamespace { module },
            arg_strings: Vec::new(),
        }
    }

    pub(in crate::plugins::analysis::mainjs::api_classifier) fn evidence_symbol(&self) -> String {
        match &self.provenance {
            MemberCallProvenance::Any | MemberCallProvenance::Rooted => self.chain.clone(),
            MemberCallProvenance::ModuleNamespace { module } => format!("{module}.{}", self.chain),
        }
    }

    fn sort_key(&self) -> (&str, &str) {
        match &self.provenance {
            MemberCallProvenance::Any => ("any", &self.chain),
            MemberCallProvenance::Rooted => ("rooted", &self.chain),
            MemberCallProvenance::ModuleNamespace { module } => (module, &self.chain),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::plugins::analysis::mainjs::api_classifier) enum MemberCallProvenance {
    Any,
    Rooted,
    ModuleNamespace { module: String },
}

impl ApiMatcher {
    pub(in crate::plugins::analysis::mainjs::api_classifier) fn is_empty(&self) -> bool {
        self.calls.is_empty()
            && self.member_calls.is_empty()
            && self.member_reads.is_empty()
            && self.imports.is_empty()
            && self.string_literals.is_empty()
            && self.classes.is_empty()
            && self.constructors.is_empty()
            && self.custom_ast.is_empty()
    }

    pub(super) fn normalized(mut self) -> Self {
        for call in &mut self.calls {
            call.name = call.name.trim().to_string();
            match &mut call.provenance {
                CallProvenance::Any | CallProvenance::Global => {}
                CallProvenance::ModuleExport { module } => *module = module.trim().to_string(),
            }
        }
        self.calls.retain(|call| {
            !call.name.is_empty()
                && match &call.provenance {
                    CallProvenance::Any | CallProvenance::Global => true,
                    CallProvenance::ModuleExport { module } => !module.is_empty(),
                }
        });
        self.calls
            .sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
        self.calls.dedup();

        for member_call in &mut self.member_calls {
            member_call.chain = normalize_member_chain(&member_call.chain);
            if let MemberCallProvenance::ModuleNamespace { module } = &mut member_call.provenance {
                *module = module.trim().to_string();
            }
        }
        self.member_calls.retain(|call| {
            !call.chain.is_empty()
                && match &call.provenance {
                    MemberCallProvenance::Any | MemberCallProvenance::Rooted => true,
                    MemberCallProvenance::ModuleNamespace { module } => !module.is_empty(),
                }
        });
        self.member_calls
            .sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
        self.member_calls.dedup();

        normalize_member_chains(&mut self.member_reads);
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

fn normalize_member_chains(values: &mut Vec<String>) {
    values.retain(|value| !value.trim().is_empty());
    for value in values.iter_mut() {
        *value = normalize_member_chain(value);
    }
    values.retain(|value| !value.is_empty());
    values.sort();
    values.dedup();
}

fn normalize_member_chain(value: &str) -> String {
    value
        .split('.')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(".")
}
