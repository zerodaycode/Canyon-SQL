/// Declares functions that takes care to deserialize data incoming
/// from some supported database in Canyon-SQL into a user's defined
/// type `T`
pub trait RowMapper<T>: Sized {
    #[cfg(feature = "postgres")]
    fn deserialize_postgresql(row: &tokio_postgres::Row) -> T;
    #[cfg(feature = "mssql")]
    fn deserialize_sqlserver(row: &tiberius::Row) -> T;
    #[cfg(feature = "mysql")]
    fn deserialize_mysql(row: &mysql_async::Row) -> T;
}
