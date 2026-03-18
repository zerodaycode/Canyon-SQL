use crate::query::querybuilder::syntax::dialect::MsSql;
use crate::query::querybuilder::syntax::emitter::SqlEmitter;
use crate::query::querybuilder::syntax::tokens::SqlTokens;

#[derive(Default)]
pub struct SqlServerEmitter<'a> {
    tokens: SqlTokens<'a>,
}

impl<'a> SqlEmitter<'a> for SqlServerEmitter<'a> {
    type Dialect = MsSql;
}
