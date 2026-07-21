use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::dialect::MySql;
use crate::query::querybuilder::syntax::emitter::types::select::select_default_plan;
use crate::query::querybuilder::syntax::emitter::{EmitStep, SqlEmitter};

#[derive(Default)]
pub struct MySqlEmitter {}
impl<'a> SqlEmitter<'a, SelectAst<'a>> for MySqlEmitter {
    type Dialect = MySql;

    const PLAN: &'a [EmitStep<'a, SelectAst<'a>>] = select_default_plan!(Self::Dialect);
}
