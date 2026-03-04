use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::emitter::types::helpers;
use crate::query::querybuilder::syntax::emitter::{AstProcessor, SqlEmitter};
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::tokens::SqlTokens;

/// Blanket implementation for all the SqlEmitter implementors that are able to generate
/// SELECT like SQL clauses
impl<'a, T> EmitSelect<'a> for T
where
    T: SqlEmitter<'a>,
{
    fn emit_select(
        &mut self,
        ast: &impl AstProcessor<'a>,
        base_ast: &BaseAst<'a>,
    ) -> SqlTokens<'a> {
        let mut tokens = SqlTokens::default();

        let select_ast =
            transient::Downcast::downcast_ref::<SelectAst>(ast.as_any()).expect("Handle this");

        tokens.keyword(Keyword::Select);

        emit_columns(select_ast, &mut tokens);
        emit_from(base_ast, &mut tokens);

        tokens
    }
}

/// DOC me this
pub trait EmitSelect<'a>: SqlEmitter<'a> {
    fn emit_select(&mut self, ast: &impl AstProcessor<'a>, base_ast: &BaseAst<'a>)
    -> SqlTokens<'a>;
}

pub(crate) fn emit_columns<'a>(ast: &SelectAst<'a>, tokens: &mut SqlTokens<'a>) {
    helpers::emit_columns(&ast.columns, tokens)
}

pub(crate) fn emit_from<'a>(base_ast: &BaseAst<'a>, tokens: &mut SqlTokens<'a>) {
    //meta.to_tokens(emitter.tokens())
    todo!()
}
