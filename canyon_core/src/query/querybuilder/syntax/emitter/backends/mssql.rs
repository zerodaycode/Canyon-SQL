use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::dialect::MsSql;
use crate::query::querybuilder::syntax::emitter::types::select::select_default_plan;
use crate::query::querybuilder::syntax::emitter::{EmitStep, SqlEmitter};

#[derive(Default)]
pub struct SqlServerEmitter {}

impl<'a> SqlEmitter<'a, SelectAst<'a>> for SqlServerEmitter {
    type Dialect = MsSql;
    const PLAN: &'a [EmitStep<'a, SelectAst<'a>>] = select_default_plan!(Self::Dialect);
}
