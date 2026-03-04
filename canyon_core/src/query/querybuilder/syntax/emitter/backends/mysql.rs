use crate::query::querybuilder::syntax::dialect::MySql;
use crate::query::querybuilder::syntax::emitter::SqlEmitter;
use crate::query::querybuilder::syntax::tokens::SqlTokens;

#[derive(Default)]
pub struct MySqlEmitter<'a> {
    tokens: SqlTokens<'a>,
}

impl<'a> SqlEmitter<'a> for MySqlEmitter<'a> {
    type Dialect = MySql;
}
