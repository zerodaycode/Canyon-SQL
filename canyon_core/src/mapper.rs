//! The mapper module of Canyon-SQL.
//!
//! This module defines traits and utilities for mapping database query results to user-defined
//! types. It includes the `RowMapper` trait and related functionality for deserialization.

pub use crate::error::{CanyonError, CanyonResult, MappingError};

/// Declares functions that takes care to deserialize data incoming
/// from some supported database in Canyon-SQL into a user's defined
/// type `T`
pub trait RowMapper: Sized {
    type Output;

    #[cfg(feature = "postgres")]
    fn deserialize_postgresql(
        row: &tokio_postgres::Row,
    ) -> CanyonResult<<Self as RowMapper>::Output>;
    #[cfg(feature = "mssql")]
    fn deserialize_sqlserver(row: &tiberius::Row) -> CanyonResult<<Self as RowMapper>::Output>;
    #[cfg(feature = "mysql")]
    fn deserialize_mysql(row: &mysql_async::Row) -> CanyonResult<<Self as RowMapper>::Output>;
}

pub trait DefaultRowMapper {
    type Mapper: RowMapper;
}

// Blanket impl to make `Mapper = Self` for any `T: RowMapper`
impl<T> DefaultRowMapper for T
where
    T: RowMapper,
{
    type Mapper = T;
}

pub trait IntoResults {
    fn into_results<R>(self) -> CanyonResult<Vec<R>>
    where
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>;
}
