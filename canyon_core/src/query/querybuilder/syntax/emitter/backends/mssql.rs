use crate::query::querybuilder::syntax::dialect::MsSql;
use crate::query::querybuilder::syntax::emitter::SqlEmitter;

#[derive(Default)]
pub struct SqlServerEmitter {}

impl SqlEmitter<'_> for SqlServerEmitter {
    type Dialect = MsSql;
}
