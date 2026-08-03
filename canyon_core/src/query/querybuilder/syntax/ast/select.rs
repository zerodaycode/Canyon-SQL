use crate::query::querybuilder::syntax::{
    column::ColumnRef, emitter::AstProcessor, having::HavingClause, join::JoinClause,
    order::OrderByClause, query_kind::QueryKind,
};

/// Structured representation of a `SELECT` statement.
///
/// `SelectAst` stores the clauses and modifiers that are specific to selection
/// queries.
///
/// NOTE: The target table and filtering conditions are held separately by
/// the shared base AST.
#[derive(Default)]
pub struct SelectAst<'a> {
    pub columns: Vec<ColumnRef<'a>>,
    /// Indicates whether the projection must emit a row count.
    pub is_count_query: bool,
    /// Indicates whether the query must emit `SELECT DISTINCT`.
    pub with_distinct: bool,
    /// Join clauses, preserved in insertion order.
    pub joins: Vec<JoinClause<'a>>,
    pub order_by: Option<OrderByClause<'a>>,
    pub having: Option<HavingClause<'a>>,
    pub group_by: Option<Vec<ColumnRef<'a>>>,
    // TODO: replace the primitive value with a dedicated domain type.
    pub limit: Option<u64>,
    // TODO: replace the primitive value with a dedicated domain type.
    pub offset: Option<u64>,
}

impl<'a> SelectAst<'a> {
    /// Creates an empty `SELECT` AST with no optional clauses or modifiers.
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
    /// Identifies this AST as a `SELECT` query.
    fn query_kind(&self) -> QueryKind {
        QueryKind::Select
    }
}
