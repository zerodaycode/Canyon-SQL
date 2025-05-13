#[cfg(feature = "mssql")]
pub mod mssql;
#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "postgres")]
pub mod postgresql;

#[macro_use]
pub mod str;
pub mod database_connection;

// Apply the macro to implement DbConnection for &str and str
impl_db_connection!(str);
impl_db_connection!(&str);
