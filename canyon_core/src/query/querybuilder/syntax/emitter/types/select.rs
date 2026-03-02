use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::emitter::{AstProcessor, SqlEmitter};
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::symbol::Symbol::Comma;
use crate::query::querybuilder::syntax::tokens::{SqlTokens, ToSqlTokens};

/// Blanket implementation for all the SqlEmitter implementors that are able to generate
/// SELECT like SQL clauses
impl<'a, T> EmitSelect<'a> for T
where
    T: SqlEmitter<'a>,
{
    fn emit_select(&mut self, ast: &impl AstProcessor<'a>, base_ast: &BaseAst<'a>) {
        let select_ast = transient::Downcast::downcast_ref::<SelectAst>(ast.as_any()).expect("Handle this");
        self.tokens().keyword(Keyword::Select);

        emit_columns(self, select_ast);
        emit_from(self, base_ast);
    }
}

/// DOC me this
pub trait EmitSelect<'a>: SqlEmitter<'a> {
    fn emit_select(&mut self, ast: &impl AstProcessor<'a>, base_ast: &BaseAst<'a>);
}

pub(crate) fn emit_columns<'a>(emitter: &mut impl SqlEmitter<'a>, ast: &SelectAst<'a>) {
    let tokens = emitter.tokens();
    for (i, column) in ast.columns.iter().enumerate() {
        if i > 0 {
            tokens.symbol(Comma);
        }
        column.to_tokens(tokens);
    }
}

pub(crate) fn emit_from<'a>(emitter: &mut impl SqlEmitter<'a>, base_ast: &BaseAst<'a>) {
    //meta.to_tokens(emitter.tokens())
    todo!()
}
