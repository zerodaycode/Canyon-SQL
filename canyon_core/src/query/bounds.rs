use std::fmt::Display;

use crate::error::CanyonResult;
use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::rows::FromSqlOwnedValue;

/// Runtime metadata and field access generated for an entity.
///
/// This contract is primarily consumed by Canyon's generated CRUD operations.
/// Field collections exclude the primary key because they currently represent
/// the values and columns used by entity insertion.
pub trait EntityRuntimeInfo {
    /// Rust type used by the entity's primary-key field.
    type PrimaryKey: FromSqlOwnedValue;

    /// Schema-qualified physical table mapped by this entity.
    const TABLE: &'static str;

    /// Returns the insertable field values in declaration order.
    ///
    /// The primary-key field is excluded.
    fn field_values(&self) -> Vec<&dyn QueryParameter>;

    /// Returns the insertable columns in the same order as [`Self::field_values`].
    ///
    /// The primary-key column is excluded.
    fn field_columns() -> Vec<ColumnRef<'static>>;

    /// Returns the mapped primary-key column name, when the entity declares one.
    fn primary_key_name() -> Option<&'static str>;

    /// Returns this entity's primary-key value, when the entity declares one.
    fn primary_key_value(&self) -> Option<&dyn QueryParameter>;

    /// Assigns a primary-key value after a successful insert.
    fn set_primary_key(&mut self, value: Self::PrimaryKey) -> CanyonResult<()>;

    /// Returns the primary-key column qualified with the entity table name.
    fn primary_key_column() -> Option<ColumnRef<'static>>;
}

/// Provides the table name associated with an entity.
///
/// Consider renaming this trait if it coexists with the concrete
/// `TableMetadata` syntax type.
pub trait EntityTable: Display {
    /// Returns the physical database table represented by this value.
    fn table_name<'a>(&self) -> &'a str;
}

/// Identifies an entity field and its mapped database column.
///
/// Implementations are normally generated as an enum with one variant per
/// mapped field.
pub trait FieldIdentifier: Display {
    /// Returns the mapped database column name.
    fn as_str(&self) -> &'static str;

    /// Converts this field identifier into a typed column reference.
    fn as_column_ref(&self) -> ColumnRef<'static> {
        ColumnRef::from(self.as_str())
    }
}

/// Provides a mapped column together with the parameter value used by a query
/// condition.
pub trait FieldValueIdentifier {
    /// Returns the mapped database column associated with this value.
    fn column(&self) -> ColumnRef<'_>;

    /// Returns the value as a database query parameter.
    fn value(&self) -> &dyn QueryParameter;
}
