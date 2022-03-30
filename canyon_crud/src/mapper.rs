use std::fmt::Debug;
use tokio_postgres::Row;

use crate::crud::Transaction;

/// Sets the way of how to deserialize a custom type T
/// from a Row object retrieved from a database query
pub trait RowMapper<T: Debug + Transaction<T>>: Sized {

    /// Deserializes a database Row result into Self
    fn deserialize(row: &Row) -> T;
}