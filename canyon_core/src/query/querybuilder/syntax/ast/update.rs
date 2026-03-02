use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::emitter::AstProcessor;
use crate::query::querybuilder::syntax::query_kind::QueryKind;
use transient::Transient;

#[derive(Transient)]
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
