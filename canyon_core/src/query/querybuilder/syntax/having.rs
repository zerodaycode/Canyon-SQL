use crate::query::operators::Comp;
use crate::query::parameters::QueryParameter;

pub struct HavingClause<'a> {
    pub column: &'a str,
    pub operator: Comp,
    pub value: &'a dyn QueryParameter, // TODO: shouldn't this be a placeholder?
}

impl<'a> HavingClause<'a> {
    pub fn new(column: &'a str, operator: Comp, value: &'a dyn QueryParameter) -> Self {
        Self { column, operator, value }
    }
}