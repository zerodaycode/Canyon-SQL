pub mod select;

use crate::connection::database_type::DatabaseType;
use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::syntax::clause::ConditionClause;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::query_kind::QueryKind;

/// Base AST for common parts
#[derive(Default)] pub struct BaseAst<'a> {
    pub kind: QueryKind,
    pub table: TableMetadata<'a>,
    pub conditions: Vec<ConditionClause<'a>>,
}

impl<'a> BaseAst<'a> {
    pub fn new(kind: QueryKind, table: TableMetadata<'a>, database_type: DatabaseType) -> Self {
        Self {
            kind,
            table,
            conditions: Vec::new()
        }
    }
}

/// Select AST


pub struct InsertAst<'a> {
    pub base: BaseAst<'a>,
    pub columns: Vec<&'a str>,
    pub values: Vec<&'a dyn QueryParameter>,
}

impl<'a> InsertAst<'a> {
    pub fn new(table: TableMetadata<'a>, db: DatabaseType) -> Self {
        Self { base: BaseAst::new(QueryKind::Insert, table, db), columns: Vec::new(), values: Vec::new() }
    }
}

pub struct UpdateAst<'a> {
    pub base: BaseAst<'a>,
    pub set_clauses: Vec<(&'a str, &'a dyn QueryParameter)>,
}

impl<'a> UpdateAst<'a> {
    pub fn new(table: TableMetadata<'a>, db: DatabaseType) -> Self {
        Self { base: BaseAst::new(QueryKind::Update, table, db), set_clauses: Vec::new() }
    }
}

pub struct DeleteAst<'a> {
    pub base: BaseAst<'a>,
}

impl<'a> DeleteAst<'a> {
    pub fn new(table: TableMetadata<'a>, db: DatabaseType) -> Self {
        Self { base: BaseAst::new(QueryKind::Delete, table, db) }
    }
}
