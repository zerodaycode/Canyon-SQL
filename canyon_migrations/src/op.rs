//! Neutral schema changes, the statements a dialect turns them into, and what a dialect may look at.

use crate::model::{
    ColumnDef, ForeignKey, Ident, PrimaryKey, QualifiedName, SchemaModel, TableDef,
};
use canyon_core::query::parameters::QueryParameter;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Op {
    CreateSchema {
        schema: Ident,
    },
    CreateTable(TableDef),
    /// A new name in the same schema (`MoveTable` changes the schema).
    RenameTable {
        from: QualifiedName,
        to: Ident,
    },
    DropTable(QualifiedName),
    AddColumn {
        table: QualifiedName,
        column: ColumnDef,
    },
    RenameColumn {
        table: QualifiedName,
        from: Ident,
        to: Ident,
    },
    DropColumn {
        table: QualifiedName,
        column: Ident,
    },
    AlterColumnType {
        table: QualifiedName,
        column: ColumnDef,
    },
    SetNotNull {
        table: QualifiedName,
        column: Ident,
    },
    DropNotNull {
        table: QualifiedName,
        column: Ident,
    },
    AddPrimaryKey {
        table: QualifiedName,
        key: PrimaryKey,
    },
    DropPrimaryKey {
        table: QualifiedName,
        name: Ident,
    },
    AddIdentity {
        table: QualifiedName,
        column: Ident,
    },
    DropIdentity {
        table: QualifiedName,
        column: Ident,
    },
    AddForeignKey {
        table: QualifiedName,
        key: ForeignKey,
    },
    DropForeignKey {
        table: QualifiedName,
        name: Ident,
    },
    /// SQL Server only (`DdlDialect::needs_rebuild`): a new table, the rows copied, the
    /// old one dropped and the new one renamed. Other tables' foreign keys to it must be
    /// dropped before and added back after.
    RebuildTable {
        /// The table as it is now (with its defaults).
        from: TableDef,
        to: TableDef,
        /// `(column in from, column in to)` for every column whose rows are copied.
        column_map: Vec<(Ident, Ident)>,
    },
    /// Same table name, another schema (`None` = the default one).
    MoveTable {
        from: QualifiedName,
        to_schema: Option<Ident>,
    },
}

impl Op {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::CreateSchema { .. } => "CreateSchema",
            Self::CreateTable(_) => "CreateTable",
            Self::RenameTable { .. } => "RenameTable",
            Self::DropTable(_) => "DropTable",
            Self::AddColumn { .. } => "AddColumn",
            Self::RenameColumn { .. } => "RenameColumn",
            Self::DropColumn { .. } => "DropColumn",
            Self::AlterColumnType { .. } => "AlterColumnType",
            Self::SetNotNull { .. } => "SetNotNull",
            Self::DropNotNull { .. } => "DropNotNull",
            Self::AddPrimaryKey { .. } => "AddPrimaryKey",
            Self::DropPrimaryKey { .. } => "DropPrimaryKey",
            Self::AddIdentity { .. } => "AddIdentity",
            Self::DropIdentity { .. } => "DropIdentity",
            Self::AddForeignKey { .. } => "AddForeignKey",
            Self::DropForeignKey { .. } => "DropForeignKey",
            Self::RebuildTable { .. } => "RebuildTable",
            Self::MoveTable { .. } => "MoveTable",
        }
    }

    /// The table the op changes (for renames and moves, the old name). `None` for
    /// schema ops.
    pub fn table(&self) -> Option<&QualifiedName> {
        Some(match self {
            Self::CreateSchema { .. } => return None,
            Self::CreateTable(t) => &t.name,
            Self::RenameTable { from, .. } | Self::MoveTable { from, .. } => from,
            Self::DropTable(t) => t,
            Self::RebuildTable { from, .. } => &from.name,
            Self::AddColumn { table, .. }
            | Self::RenameColumn { table, .. }
            | Self::DropColumn { table, .. }
            | Self::AlterColumnType { table, .. }
            | Self::SetNotNull { table, .. }
            | Self::DropNotNull { table, .. }
            | Self::AddPrimaryKey { table, .. }
            | Self::DropPrimaryKey { table, .. }
            | Self::AddIdentity { table, .. }
            | Self::DropIdentity { table, .. }
            | Self::AddForeignKey { table, .. }
            | Self::DropForeignKey { table, .. } => table,
        })
    }
}

/// A value bound to a statement, never written into its text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Param {
    Text(String),
    Int(i64),
}

impl Param {
    pub fn as_query_param(&self) -> &dyn QueryParameter {
        match self {
            Self::Text(value) => value,
            Self::Int(value) => value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Statement {
    pub sql: String,
    pub params: Vec<Param>,
    /// Can run inside the plan's transaction (false for MySQL DDL, which commits implicitly).
    pub transactional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServerVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

/// What a dialect may look at while lowering an op.
#[derive(Debug, Clone, Copy)]
pub struct Ctx<'a> {
    pub server: ServerVersion,
    /// The connection's default schema (MySQL: its database), for ops on unqualified names.
    pub default_schema: &'a str,
    pub actual: &'a SchemaModel,
}

impl Ctx<'_> {
    pub fn actual_table(&self, table: &QualifiedName) -> Option<&TableDef> {
        self.actual.table(table)
    }

    pub fn actual_column(&self, table: &QualifiedName, column: &Ident) -> Option<&ColumnDef> {
        self.actual_table(table)?
            .columns
            .iter()
            .find(|c| &c.name == column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_kind_and_table() {
        let t = QualifiedName::parse("player").unwrap();
        let op = Op::DropColumn {
            table: t.clone(),
            column: Ident::new("age").unwrap(),
        };
        assert_eq!(op.kind(), "DropColumn");
        assert_eq!(op.table(), Some(&t));
        let schema = Op::CreateSchema {
            schema: Ident::new("s").unwrap(),
        };
        assert_eq!(schema.kind(), "CreateSchema");
        assert_eq!(schema.table(), None);
    }

    #[test]
    fn server_versions_compare() {
        let v = |a, b, c| ServerVersion {
            major: a,
            minor: b,
            patch: c,
        };
        assert!(v(8, 0, 16) < v(8, 4, 0));
        assert!(v(14, 0, 0) < v(17, 2, 0));
    }

    #[test]
    fn params_bind_their_own_value() {
        let text = Param::Text("x".into());
        let int = Param::Int(7);
        assert_eq!(format!("{:?}", text.as_query_param()), r#""x""#);
        assert_eq!(format!("{:?}", int.as_query_param()), "7");
    }
}
