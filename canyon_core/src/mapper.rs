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

pub type CanyonError = Box<(dyn std::error::Error + Send + Sync)>; // TODO: convert this into a
                                                                   // real error
pub trait IntoResults {
    fn into_results<T>(self) -> Result<Vec<T>, CanyonError>
    where
        T: RowMapper<T>;
}
