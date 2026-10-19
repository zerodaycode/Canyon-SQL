//! Native SQL for each backend: types both ways and DDL for every [`crate::op::Op`].

mod naming;
mod writer;

#[cfg(feature = "mssql")]
mod mssql;
#[cfg(feature = "mysql")]
pub(crate) mod mysql;
#[cfg(feature = "postgres")]
mod pg;

#[cfg(test)]
pub(crate) mod fixtures;

pub use naming::{fk_name, pk_name};
pub use writer::{DdlWriter, TypeSql};

use crate::model::{CatalogSql, ColumnDef, ColumnDefault, ForeignKey, Ident, Logical};
use crate::model::{NativeType, PrimaryKey};
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
    /// The column is computed from others; Canyon doesn't rewrite it.
    GeneratedColumn(String),
    UnsupportedType(SqlType),
}

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotForThisDialect(op) => write!(f, "`{op}` is not used by this database"),
            Self::MissingActual(what) => write!(f, "{what} is not in the database model"),
            Self::DesiredTypeUnknown(column) => write!(f, "column `{column}` has no Canyon type"),
            Self::GeneratedColumn(column) => write!(
                f,
                "column `{column}` is generated from other columns: Canyon doesn't change it"
            ),
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

/// `actual` as a statement that changes its type to `new_type` has to write it again:
/// - the collation only for text (SQL Server refuses `COLLATE` on an `int`);
/// - a `CURRENT_TIMESTAMP` default and `ON UPDATE` moved to a timestamp's new precision
///   (MySQL wants one precision in the whole column), `ON UPDATE` dropped on any other
///   type (MySQL allows it only on timestamps);
/// - the comment and the rest as they are.
pub(crate) fn restated(actual: &ColumnDef, new_type: SqlType) -> ColumnDef {
    let precision = match new_type {
        SqlType::Timestamp { precision } => Some(precision),
        _ => None,
    };
    let mut column = actual.clone();
    column.ty = Logical::Known(new_type);
    if !matches!(new_type, SqlType::Text { .. }) {
        column.extra.collation = None;
    }
    column.extra.on_update = actual
        .extra
        .on_update
        .as_ref()
        .and(precision)
        .map(current_timestamp);
    if let (Some(ColumnDefault::Expr { constraint, sql }), Some(precision)) =
        (&actual.default, precision)
        && is_current_timestamp(sql.as_str())
    {
        column.default = Some(ColumnDefault::Expr {
            constraint: constraint.clone(),
            sql: current_timestamp(precision),
        });
    }
    column
}

/// `CURRENT_TIMESTAMP` or `CURRENT_TIMESTAMP(n)`, as the catalogs report it.
pub(crate) fn is_current_timestamp(sql: &str) -> bool {
    const KEYWORD: &str = "CURRENT_TIMESTAMP";
    let Some(rest) = sql
        .get(..KEYWORD.len())
        .filter(|head| head.eq_ignore_ascii_case(KEYWORD))
        .map(|_| &sql[KEYWORD.len()..])
    else {
        return false;
    };
    rest.is_empty()
        || rest
            .strip_prefix('(')
            .and_then(|r| r.strip_suffix(')'))
            .is_some_and(|digits| digits.bytes().all(|b| b.is_ascii_digit()))
}

/// `CURRENT_TIMESTAMP` with `precision` fractional digits, written the way MySQL reports it.
fn current_timestamp(precision: u8) -> CatalogSql {
    CatalogSql::new(match precision {
        0 => "CURRENT_TIMESTAMP".to_owned(),
        p => format!("CURRENT_TIMESTAMP({p})"),
    })
}

/// Rewriting a generated column would turn it into a plain one (MySQL) or fail (SQL Server).
#[cfg(any(feature = "mssql", feature = "mysql"))]
pub(crate) fn rewritable(column: &ColumnDef) -> Result<(), LowerError> {
    if column.extra.generated {
        Err(LowerError::GeneratedColumn(column.name.to_string()))
    } else {
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CatalogSql, ColumnExtra, TableDef};

    fn stamp(default: &str, on_update: Option<&str>) -> ColumnDef {
        let mut column = TableDef::builder("T", "t")
            .column("ts", SqlType::Timestamp { precision: 6 }, true)
            .build()
            .unwrap()
            .columns
            .remove(0);
        column.default = Some(ColumnDefault::Expr {
            constraint: None,
            sql: CatalogSql::new(default),
        });
        column.extra = ColumnExtra {
            collation: Some(CatalogSql::new("latin1_bin")),
            comment: Some("kept".into()),
            on_update: on_update.map(CatalogSql::new),
            ..ColumnExtra::default()
        };
        column
    }

    fn default_sql(c: &ColumnDef) -> Option<&str> {
        match &c.default {
            Some(ColumnDefault::Expr { sql, .. }) => Some(sql.as_str()),
            _ => None,
        }
    }

    #[test]
    fn a_new_timestamp_precision_moves_current_timestamp_along() {
        let actual = stamp("CURRENT_TIMESTAMP(6)", Some("CURRENT_TIMESTAMP(6)"));
        let to3 = restated(&actual, SqlType::Timestamp { precision: 3 });
        assert_eq!(default_sql(&to3), Some("CURRENT_TIMESTAMP(3)"));
        assert_eq!(
            to3.extra.on_update.unwrap().as_str(),
            "CURRENT_TIMESTAMP(3)"
        );
        let to0 = restated(&actual, SqlType::Timestamp { precision: 0 });
        assert_eq!(default_sql(&to0), Some("CURRENT_TIMESTAMP"));
        let legacy = stamp("CURRENT_TIMESTAMP", Some("CURRENT_TIMESTAMP"));
        let to6 = restated(&legacy, SqlType::Timestamp { precision: 6 });
        assert_eq!(default_sql(&to6), Some("CURRENT_TIMESTAMP(6)"));
        assert_eq!(
            to6.extra.on_update.unwrap().as_str(),
            "CURRENT_TIMESTAMP(6)"
        );
    }

    #[test]
    fn other_types_keep_what_they_can_hold() {
        let actual = stamp("CURRENT_TIMESTAMP(6)", Some("CURRENT_TIMESTAMP(6)"));
        let text = restated(&actual, SqlType::Text { max_len: Some(40) });
        assert_eq!(
            default_sql(&text),
            Some("CURRENT_TIMESTAMP(6)"),
            "the dialect decides"
        );
        assert_eq!(text.extra.on_update, None);
        assert!(text.extra.collation.is_some());
        let int = restated(&actual, SqlType::BigInt);
        assert_eq!(int.extra.collation, None);
        assert_eq!(int.extra.comment.as_deref(), Some("kept"));
        assert_eq!(int.ty, Logical::Known(SqlType::BigInt));
        let plain = stamp("now()", None);
        let to3 = restated(&plain, SqlType::Timestamp { precision: 3 });
        assert_eq!(default_sql(&to3), Some("now()"));
        assert_eq!(to3.extra.on_update, None);
    }

    #[test]
    fn current_timestamp_forms() {
        for (sql, expected) in [
            ("CURRENT_TIMESTAMP", true),
            ("CURRENT_TIMESTAMP(6)", true),
            ("current_timestamp(3)", true),
            ("CURRENT_TIMESTAMP()", true),
            ("CURRENT_TIMESTAMP(6) + INTERVAL 1 DAY", false),
            ("now(6)", false),
            ("CURRENT_TIMESTAMPX", false),
            ("", false),
        ] {
            assert_eq!(is_current_timestamp(sql), expected, "{sql}");
        }
    }
}
