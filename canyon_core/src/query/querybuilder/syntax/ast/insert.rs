pub(crate) use crate::query::querybuilder::syntax::{
    column::ColumnRef, emitter::AstProcessor, query_kind::QueryKind,
};

/// Structured representation of a `INSERT` statement.
#[derive(Default)]
pub struct InsertAst<'a> {
    pub columns: Vec<ColumnRef<'a>>,
    pub returning_columns: Vec<ColumnRef<'a>>,
}

impl<'a> AstProcessor<'a> for InsertAst<'a> {
    fn query_kind(&self) -> QueryKind {
        QueryKind::Insert
    }
}

impl<'a> InsertAst<'a> {
    pub const fn new() -> Self {
        Self {
            columns: Vec::new(),
            returning_columns: Vec::new(),
        }
    }
}
