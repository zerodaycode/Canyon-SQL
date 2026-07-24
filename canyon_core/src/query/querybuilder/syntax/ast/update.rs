use crate::query::querybuilder::syntax::{
    column::ColumnRef, emitter::AstProcessor, query_kind::QueryKind,
};

pub struct UpdateAst<'a> {
    pub columns: Vec<ColumnRef<'a>>,
}

impl<'a> AstProcessor<'a> for UpdateAst<'a> {
    fn query_kind(&self) -> QueryKind {
        QueryKind::Update
    }
}

impl<'a> Default for UpdateAst<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> UpdateAst<'a> {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }
}
