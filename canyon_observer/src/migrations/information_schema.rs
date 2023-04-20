#[cfg(feature = "mssql")]
use canyon_connection::tiberius::ColumnType as TIB_TY;
#[cfg(feature = "postgres")]
use canyon_connection::tokio_postgres::types::Type as TP_TYP;
use canyon_crud::bounds::{Column, ColumnType, Row, RowOperations};

/// Model that represents the database entities that belongs to the current schema.
///
/// Basically, it's an agrupation of rows of results when Canyon queries the `information schema`
/// table, grouping by table name (one [`TableMetadata`] is the rows that contains the information
/// of a table)
#[derive(Debug)]
pub struct TableMetadata {
    pub table_name: String,
    pub columns: Vec<ColumnMetadata>,
}

/// Represents the *metadata* associated with a column that belongs to a `PostgreSQL` table.
#[derive(Debug, Default)]
pub struct ColumnMetadata {
    pub column_name: String,
    pub datatype: String,
    pub character_maximum_length: Option<i32>,
    pub is_nullable: bool, // Care, postgres type is varchar
    pub column_default: Option<String>,
    pub foreign_key_info: Option<String>,
    pub foreign_key_name: Option<String>,
    pub primary_key_info: Option<String>,
    pub primary_key_name: Option<String>,
    pub is_identity: bool, // Care, postgres type is varchar
    pub identity_generation: Option<String>,
}

/// Represents the relation between a real value stored inside a [`ColumnMetadata`]
/// and the datatype of that value
#[derive(Debug)]
pub enum ColumnMetadataTypeValue {
    StringValue(Option<String>),
    IntValue(Option<i32>),
    NoneValue,
}
impl ColumnMetadataTypeValue {
    /// Retrieves the value stored in a [`Column`] for a passed [`Row`]
    pub fn get_value(row: &dyn Row, col: &Column) -> Self {
        match col.column_type() {
            #[cfg(feature = "postgres")]
            ColumnType::Postgres(v) => {
                match *v {
                    TP_TYP::NAME | TP_TYP::VARCHAR | TP_TYP::TEXT => Self::StringValue(
                        row.get_postgres_opt::<&str>(col.name())
                            .map(|opt| opt.to_owned()),
                    ),
                    TP_TYP::INT4 => Self::IntValue(row.get_postgres_opt::<i32>(col.name())),
                    _ => Self::NoneValue, // TODO watchout this one
                }
            }
            #[cfg(feature = "mssql")]
            ColumnType::SqlServer(v) => match v {
                TIB_TY::NChar | TIB_TY::NVarchar | TIB_TY::BigChar | TIB_TY::BigVarChar => {
                    Self::StringValue(
                        row.get_mssql_opt::<&str>(col.name())
                            .map(|opt| opt.to_owned()),
                    )
                }
                TIB_TY::Int2 | TIB_TY::Int4 | TIB_TY::Int8 | TIB_TY::Intn => {
                    Self::IntValue(row.get_mssql_opt::<i32>(col.name()))
                }
                _ => Self::NoneValue,
            },
        }
    }
}
