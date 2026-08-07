use crate::query::querybuilder::syntax::{emitter::AstProcessor, query_kind::QueryKind};

/// Structured representation of a `DELETE` statement.
#[derive(Default)]
pub struct DeleteAst {}

impl<'a> AstProcessor<'a> for DeleteAst {
    fn query_kind(&self) -> QueryKind {
        QueryKind::Delete
    }
}

impl DeleteAst {
    pub const fn new() -> Self {
        Self {}
    }
}
