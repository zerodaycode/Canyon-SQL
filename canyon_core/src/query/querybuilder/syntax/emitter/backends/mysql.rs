use crate::query::querybuilder::syntax::{
    ast::{delete::DeleteAst, insert::InsertAst, select::SelectAst, update::UpdateAst},
    dialect::MySql,
    emitter::{
        EmitStep, SqlEmitter,
        types::{
            delete::delete_default_plan, insert::insert_default_plan, select::select_default_plan,
            update::update_default_plan,
        },
    },
};

#[derive(Default)]
pub struct MySqlEmitter {}

impl<'a> SqlEmitter<'a, SelectAst<'a>> for MySqlEmitter {
    type Dialect = MySql;

    const PLAN: &'a [EmitStep<'a, SelectAst<'a>>] = select_default_plan!(Self::Dialect);
}

impl<'a> SqlEmitter<'a, InsertAst<'a>> for MySqlEmitter {
    type Dialect = MySql;

    const PLAN: &'a [EmitStep<'a, InsertAst<'a>>] = insert_default_plan!(Self::Dialect);
}

impl<'a> SqlEmitter<'a, UpdateAst<'a>> for MySqlEmitter {
    type Dialect = MySql;

    const PLAN: &'a [EmitStep<'a, UpdateAst<'a>>] = update_default_plan!(Self::Dialect);
}

impl<'a> SqlEmitter<'a, DeleteAst> for MySqlEmitter {
    type Dialect = MySql;

    const PLAN: &'a [EmitStep<'a, DeleteAst>] = delete_default_plan!(Self::Dialect);
}
