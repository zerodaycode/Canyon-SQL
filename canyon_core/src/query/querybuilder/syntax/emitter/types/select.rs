use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::emitter::SqlEmitter;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::symbol::Symbol::Comma;
use crate::query::querybuilder::syntax::tokens::{SqlTokens, ToSqlTokens};

/// Blanket implementation for all the SqlEmitter implementors that are able to generate
/// SELECT like SQL clauses
impl<'a, T> EmitSelect<'a> for T
where
    T: SqlEmitter<'a, SelectAst<'a>> + 'a
{
    fn emit_select(
        &mut self,
        ast: &'a SelectAst<'a>,
        base_ast: &'a BaseAst<'a>
    ) {
        self.tokens().keyword(Keyword::Select);

        emit_columns(self, ast);
        emit_from(self, base_ast);
    }
}

/// DOC me this
pub trait EmitSelect<'a>: SqlEmitter<'a, SelectAst<'a>> + Sized where Self: 'a {
    fn emit_select(
        &mut self,
        ast: &'a SelectAst,
        base_ast: &'a BaseAst<'a>
    );
}

pub(crate) fn emit_columns<'a>(emitter: &mut impl SqlEmitter<'a, SelectAst<'a>>, ast: &SelectAst<'a>) {
    let tokens = emitter.tokens();
    for (i, column) in ast.columns.iter().enumerate() {
        if i > 0 {
            tokens.symbol(Comma);
        }
        column.to_tokens(tokens);
    }
}

pub(crate) fn emit_from<'a>(emitter: &mut impl SqlEmitter<'a, SelectAst<'a>>, base_ast: &'a BaseAst) {
    //meta.to_tokens(emitter.tokens())
    todo!()
}