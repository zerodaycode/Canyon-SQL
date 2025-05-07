pub mod mssql;
pub mod mysql;
pub mod postgresql;

#[macro_use]
pub mod str;
pub mod database_connection;

// Apply the macro to implement DbConnection for &str and str
impl_db_connection!(str);
impl_db_connection!(&str);
