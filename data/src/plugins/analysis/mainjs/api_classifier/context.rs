use swc_ecma_ast::Program;

use super::custom_matchers::SemanticIndex;
use super::symbol_index::AliasInfo;

pub(super) struct ApiMatchContext<'a> {
    pub(super) program: Option<&'a Program>,
    pub(super) aliases: AliasInfo,
    pub(super) semantics: SemanticIndex,
}

impl<'a> ApiMatchContext<'a> {
    pub(super) fn new(program: Option<&'a Program>) -> Self {
        let aliases = program.map(AliasInfo::collect).unwrap_or_default();
        let semantics = program
            .map(|program| SemanticIndex::collect(program, &aliases))
            .unwrap_or_default();
        Self {
            program,
            aliases,
            semantics,
        }
    }
}
