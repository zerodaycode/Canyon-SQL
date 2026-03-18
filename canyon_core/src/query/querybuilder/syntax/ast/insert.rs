use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::emitter::AstProcessor;
use crate::query::querybuilder::syntax::query_kind::QueryKind;
use transient::Transient;

#[derive(Default, Transient)]
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
