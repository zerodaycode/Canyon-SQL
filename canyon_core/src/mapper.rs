/// Declares functions that takes care to deserialize data incoming
/// from some supported database in Canyon-SQL into a user's defined
/// type `T`
pub trait RowMapper: Sized {
    type Output;

    #[cfg(feature = "postgres")]
    fn deserialize_postgresql(row: &tokio_postgres::Row) -> <Self as RowMapper>::Output;
    #[cfg(feature = "mssql")]
    fn deserialize_sqlserver(row: &tiberius::Row) -> Self::Output;
    #[cfg(feature = "mysql")]
    fn deserialize_mysql(row: &mysql_async::Row) -> Self::Output;
}

pub type CanyonError = Box<(dyn std::error::Error + Send + Sync)>; // TODO: convert this into a
                                                                   // real error
pub trait IntoResults {
    fn into_results<R>(self) -> Result<Vec<R>, CanyonError>
    where
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>;
}
