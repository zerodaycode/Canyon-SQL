#[cfg(feature = "tiberius")]
use canyon_connection::tiberius;
#[cfg(feature = "tokio-postgres")]
use canyon_connection::tokio_postgres;

use crate::crud::Transaction;

/// Declares functions that takes care to deserialize data incoming
/// from some supported database in Canyon-SQL into a user's defined
/// type `T`
pub trait RowMapper<T: Transaction<T>>: Sized {
    #[cfg(feature = "tokio-postgres")]
    fn deserialize_postgresql(row: &tokio_postgres::Row) -> T;
    #[cfg(feature = "tiberius")]
    fn deserialize_sqlserver(row: &tiberius::Row) -> T;
}
