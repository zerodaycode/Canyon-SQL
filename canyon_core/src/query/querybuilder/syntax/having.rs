use crate::query::operators::Comp;
use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::syntax::column::ColumnRef;

pub struct HavingClause<'a> {
    pub column: ColumnRef<'a>,
    pub operator: Comp,
    pub value: &'a dyn QueryParameter, // TODO: shouldn't this be a placeholder?
}

impl<'a> HavingClause<'a> {
    pub fn new<I: Into<ColumnRef<'a>>>(
        column: I,
        operator: Comp,
        value: &'a dyn QueryParameter,
    ) -> Self {
        Self {
            column: column.into(),
            operator,
            value,
        }
    }
}
