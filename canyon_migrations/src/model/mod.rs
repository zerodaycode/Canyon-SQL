//! The schema model the migrations work on: what the code wants and what the database has.

mod error;
mod ident;
mod schema;
mod types;

pub use error::ModelError;
pub use ident::{Ident, MAX_IDENT_LEN, QualifiedName};
pub use schema::{
    ColumnDef, ColumnDefault, ColumnExtra, ForeignKey, Generated, PrimaryKey, SchemaModel,
    TableDef, TableDefBuilder,
};
pub use types::{CatalogSql, Compat, Length, Logical, NativeType, compat};
