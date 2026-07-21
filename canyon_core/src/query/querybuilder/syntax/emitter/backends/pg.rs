use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::dialect::PgDialect;
use crate::query::querybuilder::syntax::emitter::{EmitStep, SqlEmitter};
use crate::query::querybuilder::syntax::emitter::types::select::{select_default_plan};

#[derive(Default)]
pub struct PgEmitter {}

// PostgreSQL is the default SQL dialect in Canyon-SQL, so the default
// implementation is based on the postgres one, that's why postgres doesn't override it
impl<'a> SqlEmitter<'a, SelectAst<'a>> for PgEmitter {
    type Dialect = PgDialect;

    const PLAN: &'a [EmitStep<'a, SelectAst<'a>>] = select_default_plan!(Self::Dialect);
}