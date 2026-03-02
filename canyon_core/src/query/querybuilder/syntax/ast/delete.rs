use crate::query::querybuilder::syntax::emitter::AstProcessor;
use crate::query::querybuilder::syntax::query_kind::QueryKind;
use transient::Transient;

#[derive(Transient)]
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
