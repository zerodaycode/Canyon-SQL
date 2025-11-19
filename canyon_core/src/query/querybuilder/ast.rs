use crate::connection::database_type::DatabaseType;
use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::having::HavingClause;
use crate::query::querybuilder::syntax::join::JoinClause;
use crate::query::querybuilder::syntax::order::OrderByClause;
use crate::query::querybuilder::syntax::clause::ConditionClause;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::query_kind::QueryKind;

/// Base AST for common parts
pub struct BaseAst<'a> {
    pub kind: QueryKind,
    pub table: TableMetadata,
    pub conditions: Vec<ConditionClause<'a>>,
    pub params: Vec<&'a dyn QueryParameter>,
    pub database_type: DatabaseType,
}

impl<'a> BaseAst<'a> {
    pub fn new(kind: QueryKind, table: TableMetadata, database_type: DatabaseType) -> Self {
        Self {
            kind,
            table,
            conditions: Vec::new(),
            params: Vec::new(),
            database_type,
        }
    }
}

/// Select AST
pub struct SelectAst<'a> {
    pub base: BaseAst<'a>,
    pub columns: Vec<ColumnRef<'a>>,
    pub joins: Vec<JoinClause<'a>>,
    pub group_by: Vec<&'a str>,
    pub having: Vec<HavingClause<'a>>,
    pub order_by: Vec<OrderByClause<'a>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl<'a> SelectAst<'a> {
    pub fn new(table: TableMetadata, db: DatabaseType) -> Self {
        Self {
            base: BaseAst::new(QueryKind::Select, table, db),
            columns: Vec::new(),
            joins: Vec::new(),
            group_by: Vec::new(),
            having: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }
}

pub struct InsertAst<'a> {
    pub base: BaseAst<'a>,
    pub columns: Vec<&'a str>,
    pub values: Vec<&'a dyn QueryParameter>,
}

impl<'a> InsertAst<'a> {
    pub fn new(table: TableMetadata, db: DatabaseType) -> Self {
        Self { base: BaseAst::new(QueryKind::Insert, table, db), columns: Vec::new(), values: Vec::new() }
    }
}

pub struct UpdateAst<'a> {
    pub base: BaseAst<'a>,
    pub set_clauses: Vec<(&'a str, &'a dyn QueryParameter)>,
}

impl<'a> UpdateAst<'a> {
    pub fn new(table: TableMetadata, db: DatabaseType) -> Self {
        Self { base: BaseAst::new(QueryKind::Update, table, db), set_clauses: Vec::new() }
    }
}

pub struct DeleteAst<'a> {
    pub base: BaseAst<'a>,
}

impl<'a> DeleteAst<'a> {
    pub fn new(table: TableMetadata, db: DatabaseType) -> Self {
        Self { base: BaseAst::new(QueryKind::Delete, table, db) }
    }
}
