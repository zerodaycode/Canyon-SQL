use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::dialect::PgDialect;
use crate::query::querybuilder::syntax::emitter::types::select::EmitSelect;
use crate::query::querybuilder::syntax::emitter::{AstProcessor, SqlEmitter};
use crate::query::querybuilder::syntax::query_kind::QueryKind;
use crate::query::querybuilder::syntax::tokens::SqlTokens;

#[derive(Default)]
pub struct PgEmitter<'a> {
    tokens: SqlTokens<'a>,
}

// PostgreSQL is the default SQL dialect in Canyon-SQL, so the default
// implementation is based on the postgres one, that's why postgres doesn't override it
impl<'a> SqlEmitter<'a> for PgEmitter<'a> {
    type Dialect = PgDialect;

    fn tokens(&mut self) -> &mut SqlTokens<'a> {
        &mut self.tokens
    }
}
