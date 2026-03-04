use crate::query::querybuilder::syntax::ast::insert::InsertAst;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::emitter::SqlEmitter;
use crate::query::querybuilder::syntax::emitter::types::helpers;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::tokens::SqlTokens;

pub(crate) fn emit_returning<'a, E: SqlEmitter<'a>>(
    ast: &InsertAst<'a>,
    tokens: &mut SqlTokens<'a>,
) {
    if !E::Dialect::SUPPORTS_RETURNING {
        return; // early guarding
    }
    // TODO: actual impl, not the one below, is should have a RETURNING COLS on the INSERT implementation
    tokens.keyword(Keyword::Returning);
    // add the columns
    helpers::emit_columns(&ast.columns, tokens)
}
