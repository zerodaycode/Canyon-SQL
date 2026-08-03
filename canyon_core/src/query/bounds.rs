use std::error::Error;
use std::fmt::Display;

use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::rows::FromSqlOwnedValue;

/// Runtime metadata and field access generated for an entity.
///
/// This contract is primarily consumed by Canyon's generated CRUD operations.
/// Field collections exclude the primary key because they currently represent
/// the values and columns used by entity insertion.
pub trait EntityRuntimeInfo {
    type PrimaryKey: FromSqlOwnedValue<Self::PrimaryKey>;

    /// Returns the insertable field values in declaration order.
    ///
    /// The primary-key field is excluded.
    fn field_values(&self) -> Vec<&dyn QueryParameter>;

    /// Returns the insertable columns in the same order as [`Self::field_values`].
    ///
    /// The primary-key column is excluded.
    fn field_columns() -> Vec<ColumnRef<'static>>;

    fn primary_key_name() -> Option<&'static str>;

    fn primary_key_value(&self) -> Option<&dyn QueryParameter>;

    fn set_primary_key(
        &mut self,
        value: Self::PrimaryKey,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    fn primary_key_column() -> Option<ColumnRef<'static>>;
}

/// Provides the table name associated with an entity.
///
/// Consider renaming this trait if it coexists with the concrete
/// `TableMetadata` syntax type.
pub trait EntityTable: Display {
    fn table_name<'a>(&self) -> &'a str;
}

/// Identifies an entity field and its mapped database column.
///
/// Implementations are normally generated as an enum with one variant per
/// mapped field.
pub trait FieldIdentifier: Display {
    fn as_str(&self) -> &'static str;

    fn as_column_ref(&self) -> ColumnRef<'static> {
        ColumnRef::from(self.as_str())
    }
}

/// Provides a mapped column together with the parameter value used by a query
/// condition.
pub trait FieldValueIdentifier {
    fn column(&self) -> ColumnRef<'_>;

    fn value(&self) -> &dyn QueryParameter;
}

/// Provides access to the local field participating in a foreign-key relation.
///
/// `Related` identifies the entity on the referenced side of the relation,
/// allowing generated code to select the correct implementation when several
/// relationships exist.
pub trait ForeignKeyable<Related> {
    fn foreign_key_value(&self, column: &str) -> Option<&dyn QueryParameter>;
}
