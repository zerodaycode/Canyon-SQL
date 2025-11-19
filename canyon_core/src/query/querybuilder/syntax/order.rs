#[derive(Debug, Clone)]
pub struct OrderByClause<'a> {
    pub column: &'a str,
    pub descending: bool,
}

impl<'a> OrderByClause<'a> {
    pub fn asc(column: &'a str) -> Self { Self { column, descending: false } }
    pub fn desc(column: &'a str) -> Self { Self { column, descending: true } }
}
