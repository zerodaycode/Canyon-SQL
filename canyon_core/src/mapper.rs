//! The mapper module of Canyon-SQL.
//!
//! This module defines traits and utilities for mapping database query results to user-defined
//! types. It includes the `RowMapper` trait and related functionality for deserialization.

/// Declares functions that takes care to deserialize data incoming
/// from some supported database in Canyon-SQL into a user's defined
/// type `T`
pub trait RowMapper: Sized {
    type Output;

    #[cfg(feature = "postgres")]
    fn deserialize_postgresql(
        row: &tokio_postgres::Row,
    ) -> Result<<Self as RowMapper>::Output, CanyonError>;
    #[cfg(feature = "mssql")]
    fn deserialize_sqlserver(
        row: &tiberius::Row,
    ) -> Result<<Self as RowMapper>::Output, CanyonError>;
    #[cfg(feature = "mysql")]
    fn deserialize_mysql(
        row: &mysql_async::Row,
    ) -> Result<<Self as RowMapper>::Output, CanyonError>;
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

pub type CanyonError = Box<(dyn std::error::Error + Send + Sync)>; // TODO: convert this into a
                                                                   // real error
pub trait IntoResults {
    fn into_results<R>(self) -> Result<Vec<R>, CanyonError>
    where
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>;
}
