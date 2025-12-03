use crate::query::querybuilder::syntax::column::ColumnRef;

#[derive(Debug, Clone, Default)]
pub struct OrderByClause<'a> {
    pub column: ColumnRef<'a>,
    pub descending: bool,
}

impl<'a> OrderByClause<'a> {
    pub fn new<I: Into<ColumnRef<'a>>>(column: I, descending: bool) -> Self { Self { column: column.into(), descending } }
}
