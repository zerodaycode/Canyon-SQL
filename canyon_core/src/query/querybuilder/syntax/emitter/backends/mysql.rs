use crate::query::querybuilder::syntax::dialect::MySql;
use crate::query::querybuilder::syntax::emitter::SqlEmitter;

#[derive(Default)]
pub struct MySqlEmitter {}
impl SqlEmitter<'_> for MySqlEmitter {
    type Dialect = MySql;
}
