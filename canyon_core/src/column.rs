use std::{any::Any, borrow::Cow};

#[cfg(feature = "mysql")]
use mysql_async::{self};
#[cfg(feature = "mssql")]
use tiberius::{self};
#[cfg(feature = "postgres")]
use tokio_postgres::{self};

/// Generic abstraction for hold a Column type that will be one of the Column
/// types present in the dependent crates
pub struct Column<'a> {
    pub(crate) name: Cow<'a, str>,
    pub(crate) type_: NativeColumnType,
}
impl Column<'_> {
    pub fn name(&self) -> &str {
        &self.name
    }
    /// The driver's type of this column, see [`NativeColumnType`].
    pub fn column_type(&self) -> &NativeColumnType {
        &self.type_
    }
}

#[deprecated(since = "0.7.0", note = "unused, will be removed")]
pub trait ColType {
    fn as_any(&self) -> &dyn Any;
}
#[cfg(feature = "postgres")]
#[allow(deprecated)]
impl ColType for tokio_postgres::types::Type {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[cfg(feature = "mssql")]
#[allow(deprecated)]
impl ColType for tiberius::ColumnType {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[cfg(feature = "mysql")]
#[allow(deprecated)]
impl ColType for mysql_async::consts::ColumnType {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The driver's type of a column in a result row.
///
/// Not to be confused with [`crate::types::ColumnType`], the column type of an entity field.
/// Which variants exist depends on the enabled features, so a `match` needs a `_` arm.
///
/// ```ignore
/// use canyon_core::column::NativeColumnType;
///
/// for column in row.columns() {
///     match column.column_type() {
///         NativeColumnType::Postgres(ty) => println!("{}: {ty}", column.name()),
///         _ => {}
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum NativeColumnType {
    #[cfg(feature = "postgres")]
    Postgres(tokio_postgres::types::Type),
    #[cfg(feature = "mssql")]
    SqlServer(tiberius::ColumnType),
    #[cfg(feature = "mysql")]
    MySQL(mysql_async::consts::ColumnType),
}

/// Old name of [`NativeColumnType`].
///
/// `ColumnType::Postgres(..)` still works in expressions and patterns. Variant imports
/// (`use ...::ColumnType::Postgres`) don't: import them from `NativeColumnType`.
#[deprecated(
    since = "0.7.0",
    note = "renamed to `NativeColumnType`; `ColumnType` is now the field type trait (`canyon_sql::core::ColumnType`)"
)]
pub type ColumnType = NativeColumnType;

#[cfg(test)]
#[allow(deprecated, dead_code)]
mod tests {
    use super::ColumnType;

    // Compiles only while the old name works in patterns, for every backend
    fn old_name_matches(t: &ColumnType) -> &'static str {
        match t {
            #[cfg(feature = "postgres")]
            ColumnType::Postgres(_) => "postgres",
            #[cfg(feature = "mssql")]
            ColumnType::SqlServer(_) => "mssql",
            #[cfg(feature = "mysql")]
            ColumnType::MySQL(_) => "mysql",
        }
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn old_name_builds_and_matches_the_new_type() {
        let t: super::NativeColumnType = ColumnType::Postgres(tokio_postgres::types::Type::INT4);
        assert_eq!(old_name_matches(&t), "postgres");
    }
}
