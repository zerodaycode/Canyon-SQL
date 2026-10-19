//! Native SQL for each backend: types both ways and DDL for every [`crate::op::Op`].

mod naming;
mod writer;

#[cfg(feature = "mssql")]
mod mssql;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod pg;

#[cfg(test)]
pub(crate) mod fixtures;

pub use naming::{fk_name, pk_name};
pub use writer::{DdlWriter, TypeSql};

use crate::model::{ColumnDef, ColumnDefault, ForeignKey, Ident, Logical, NativeType, PrimaryKey};
use crate::model::{QualifiedName, TableDef};
use crate::op::{Ctx, Op, Statement};
use canyon_core::query::querybuilder::syntax::dialect::SqlDialect;
use canyon_core::types::SqlType;
use std::fmt;

/// DDL for one backend. Implemented for the query builder's dialects.
pub trait DdlDialect: SqlDialect + Sized {
    /// DDL can run inside the plan's transaction (MySQL commits it implicitly).
    const TRANSACTIONAL_DDL: bool;

    /// The column type Canyon creates for `ty`.
    fn type_sql(ty: SqlType) -> Result<TypeSql, LowerError>;

    /// A catalog type back to Canyon's types; `Other` when there is no mapping.
    fn to_logical(native: &NativeType) -> Logical;

    /// Ops this backend can only do by rebuilding the table.
    fn needs_rebuild(_op: &Op) -> bool {
        false
    }

    /// `value` as an escaped string literal of this backend.
    fn text_literal(value: &str) -> String;

    /// The SQL after `DEFAULT` for a default read from this database.
    fn default_sql(w: &mut DdlWriter<Self>, default: &ColumnDefault) {
        match default {
            ColumnDefault::Expr { sql, .. } => w.catalog_sql(sql),
            ColumnDefault::Literal { value, .. } => w.literal(value),
        };
    }

    /// The statements for `op`, in order.
    fn lower(op: &Op, ctx: &Ctx<'_>) -> Result<Vec<Statement>, LowerError>;
}

/// A request the planner should never make; returned instead of panicking.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LowerError {
    NotForThisDialect(&'static str),
    MissingActual(String),
    DesiredTypeUnknown(String),
    UnsupportedType(SqlType),
}

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotForThisDialect(op) => write!(f, "`{op}` is not used by this database"),
            Self::MissingActual(what) => write!(f, "{what} is not in the database model"),
            Self::DesiredTypeUnknown(column) => write!(f, "column `{column}` has no Canyon type"),
            Self::UnsupportedType(ty) => {
                write!(f, "{ty:?} has no column type on this database yet")
            }
        }
    }
}

impl std::error::Error for LowerError {}

pub(crate) fn one<D: DdlDialect>(w: DdlWriter<D>) -> Result<Vec<Statement>, LowerError> {
    Ok(vec![w.finish()])
}

pub(crate) fn known(column: &ColumnDef) -> Result<SqlType, LowerError> {
    match &column.ty {
        Logical::Known(ty) => Ok(*ty),
        Logical::Other(_) => Err(LowerError::DesiredTypeUnknown(column.name.to_string())),
    }
}

/// The type a desired column asks for.
pub(crate) fn desired_type<D: DdlDialect>(column: &ColumnDef) -> Result<TypeSql, LowerError> {
    D::type_sql(known(column)?)
}

/// The column's type as the database has it, so a change that restates the column
/// keeps a "compatible" type as it is.
#[cfg(any(feature = "mssql", feature = "mysql"))]
pub(crate) fn actual_type<D: DdlDialect>(column: &ColumnDef) -> Result<TypeSql, LowerError> {
    match &column.ty {
        Logical::Known(ty) => D::type_sql(*ty),
        Logical::Other(native) => Ok(TypeSql::from_catalog(&native.raw)),
    }
}

#[cfg(any(feature = "mssql", feature = "mysql"))]
pub(crate) fn actual_column<'a>(
    ctx: &'a Ctx<'_>,
    table: &QualifiedName,
    column: &Ident,
) -> Result<(&'a TableDef, &'a ColumnDef), LowerError> {
    let missing = || LowerError::MissingActual(format!("column {table}.{column}"));
    let t = ctx.actual_table(table).ok_or_else(missing)?;
    let c = t
        .columns
        .iter()
        .find(|c| &c.name == column)
        .ok_or_else(missing)?;
    Ok((t, c))
}

pub(crate) fn add_primary_key<D: DdlDialect>(
    table: &QualifiedName,
    key: &PrimaryKey,
) -> DdlWriter<D> {
    let name = key.name.clone().unwrap_or_else(|| pk_name(table));
    let mut w = DdlWriter::new();
    w.raw("ALTER TABLE ")
        .qualified(table)
        .raw(" ADD CONSTRAINT ")
        .ident(&name)
        .raw(" PRIMARY KEY (")
        .idents(&key.columns)
        .raw(")");
    w
}

pub(crate) fn add_foreign_key<D: DdlDialect>(
    table: &QualifiedName,
    name: &Ident,
    key: &ForeignKey,
    ref_table: &QualifiedName,
) -> DdlWriter<D> {
    let mut w = DdlWriter::new();
    w.raw("ALTER TABLE ")
        .qualified(table)
        .raw(" ADD CONSTRAINT ")
        .ident(name)
        .raw(" FOREIGN KEY (")
        .idents(&key.columns)
        .raw(") REFERENCES ")
        .qualified(ref_table)
        .raw(" (")
        .idents(&key.ref_columns)
        .raw(")");
    w
}

pub(crate) fn drop_constraint<D: DdlDialect>(table: &QualifiedName, name: &Ident) -> DdlWriter<D> {
    let mut w = DdlWriter::new();
    w.raw("ALTER TABLE ")
        .qualified(table)
        .raw(" DROP CONSTRAINT ")
        .ident(name);
    w
}

/// The constraints of `actual` that carry the name Canyon gives them, with the name
/// they get once the table is called `to`. Constraints named by hand keep their name.
pub(crate) struct RenamedConstraints<'a> {
    pub primary_key: Option<(Ident, Ident)>,
    pub foreign_keys: Vec<(&'a ForeignKey, Ident, Ident)>,
}

pub(crate) fn renamed_constraints<'a>(
    actual: Option<&'a TableDef>,
    from: &QualifiedName,
    to: &QualifiedName,
) -> RenamedConstraints<'a> {
    let Some(actual) = actual else {
        return RenamedConstraints {
            primary_key: None,
            foreign_keys: Vec::new(),
        };
    };
    let primary_key = actual
        .primary_key
        .as_ref()
        .and_then(|pk| pk.name.clone())
        .filter(|name| *name == pk_name(from))
        .map(|name| (name, pk_name(to)));
    let foreign_keys = actual
        .foreign_keys
        .iter()
        .filter_map(|fk| {
            let name = fk.name.clone()?;
            (name == fk_name(from, &fk.columns)).then(|| (fk, name, fk_name(to, &fk.columns)))
        })
        .collect();
    RenamedConstraints {
        primary_key,
        foreign_keys,
    }
}
