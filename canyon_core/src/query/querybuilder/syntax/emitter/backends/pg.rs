use crate::query::querybuilder::syntax::dialect::PgDialect;
use crate::query::querybuilder::syntax::emitter::SqlEmitter;

#[derive(Default)]
pub struct PgEmitter {}

// PostgreSQL is the default SQL dialect in Canyon-SQL, so the default
// implementation is based on the postgres one, that's why postgres doesn't override it
impl SqlEmitter<'_> for PgEmitter {
    type Dialect = PgDialect;
}
