use crate::query::querybuilder::syntax::ast::insert::InsertAst;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::emitter::SqlEmitter;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::symbol::Symbol::Comma;
use crate::query::querybuilder::syntax::tokens::ToSqlTokens;

pub(crate) fn emit_returning<'a, E: SqlEmitter<'a>>(emitter: &mut E, ast: &InsertAst<'a>) {
    if !E::Dialect::SUPPORTS_RETURNING {
        return; // early guarding
    }
    // TODO: actual impl, not the one below, is should have a RETURNING COLS on the INSERT implementation
    let tokens_holder = emitter.tokens();
    tokens_holder.keyword(Keyword::Returning);

    for (i, column) in ast.columns.iter().enumerate() {
        // TODO:: this is duplicated
        if i > 0 {
            tokens_holder.symbol(Comma);
        }
        column.to_tokens(tokens_holder); // TODO: strange
    }
}
