use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::order::OrderByClause;
use crate::query::querybuilder::syntax::query_kind::QueryKind;
use crate::query::querybuilder::syntax::{
    emitter::AstProcessor, having::HavingClause, join::JoinClause,
};
use transient::Transient;

#[derive(Default, Transient)]
pub struct SelectAst<'a> {
    pub columns: Vec<ColumnRef<'a>>,
    pub is_count_query: bool,
    pub with_distinct: bool,
    pub joins: Vec<JoinClause<'a>>,
    pub order_by: Option<OrderByClause<'a>>,
    pub having: Option<HavingClause<'a>>,
    pub group_by: Option<Vec<ColumnRef<'a>>>,
    pub limit: Option<u64>, // TODO: strong typing
    pub offset: Option<u64>,
}

impl<'a> SelectAst<'a> {
    pub const fn new() -> Self {
        Self {
            columns: Vec::new(),
            is_count_query: false,
            with_distinct: false,
            joins: Vec::new(),
            order_by: None,
            group_by: None,
            having: None,
            limit: None,
            offset: None,
        }
    }
}

impl<'a> AstProcessor<'a> for SelectAst<'a> {
    fn query_kind(&self) -> QueryKind {
        QueryKind::Select
    }
}
