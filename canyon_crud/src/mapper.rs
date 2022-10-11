use std::fmt::Debug;
use canyon_connection::{tokio_postgres, tiberius};

use crate::crud::Transaction;

/// Sets the way of how to deserialize a custom type T
/// from a Row object retrieved from a database query
pub trait RowMapper<T: Debug + Transaction<T>>: Sized {

    /// Deserializes a database Row result into Self
    fn deserialize_postgresql(row: &tokio_postgres::Row) -> T;

    fn deserialize_sqlserver(row: &tiberius::Row) -> T;
}