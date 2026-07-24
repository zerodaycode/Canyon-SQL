use crate::query::querybuilder::syntax::{emitter::AstProcessor, query_kind::QueryKind};

pub struct DeleteAst {}

impl<'a> AstProcessor<'a> for DeleteAst {
    fn query_kind(&self) -> QueryKind {
        QueryKind::Delete
    }
}

impl Default for DeleteAst {
    fn default() -> Self {
        Self::new()
    }
}

impl DeleteAst {
    pub fn new() -> Self {
        Self {}
    }
}
