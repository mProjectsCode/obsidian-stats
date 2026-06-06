use swc_ecma_ast::{CallExpr, Expr, Lit, MemberExpr, Program};

use super::rule::MemberCallProvenance;
use super::symbol_index::{AliasInfo, SymbolMemberProvenance, member_chain};

pub(super) struct ApiMatchContext<'a> {
    pub(super) program: Option<&'a Program>,
    pub(super) aliases: AliasInfo,
}

impl<'a> ApiMatchContext<'a> {
    pub(super) fn new(_source: &'a str, program: Option<&'a Program>) -> Self {
        let aliases = program.map(AliasInfo::collect).unwrap_or_default();
        Self { program, aliases }
    }

    pub(super) fn member_chain(&self, member: &MemberExpr) -> Option<String> {
        member_chain(member)
    }

    pub(super) fn literal_arg(&self, call: &CallExpr, index: usize) -> Option<String> {
        let arg = call.args.get(index)?;
        self.literal_string(&arg.expr)
    }

    pub(super) fn literal_string(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Lit(Lit::Str(value)) => Some(value.value.to_string_lossy().to_string()),
            Expr::Tpl(tpl) if tpl.exprs.is_empty() && tpl.quasis.len() == 1 => {
                tpl.quasis.first().map(|quasi| quasi.raw.to_string())
            }
            Expr::Paren(paren) => self.literal_string(&paren.expr),
            _ => None,
        }
    }

    pub(super) fn is_member_call_match(
        &self,
        member: &MemberExpr,
        chain: &str,
        provenance: &MemberCallProvenance,
    ) -> bool {
        match provenance {
            MemberCallProvenance::Any => self.member_chain(member).as_deref() == Some(chain),
            MemberCallProvenance::ModuleNamespace { module } => {
                matches!(
                    self.aliases.member_call_provenance(member),
                    Some(SymbolMemberProvenance::ModuleNamespace {
                        module: found_module,
                        member
                    }) if found_module == *module && member == chain
                )
            }
        }
    }
}
