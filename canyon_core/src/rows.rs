#![allow(unreachable_patterns)]

//! The rows module of Canyon-SQL.
//!
//! This module defines the `CanyonRows` enum, which wraps database query results for supported
//! databases. It also provides traits and utilities for mapping rows to user-defined types.

#[cfg(feature = "mysql")]
use mysql_async::{self};
#[cfg(feature = "mssql")]
use tiberius::{self};
#[cfg(feature = "postgres")]
use tokio_postgres::{self};

use crate::mapper::RowMapper;
use crate::row::Row;

/// Lightweight wrapper over the collection of results of the different crates
/// supported by Canyon-SQL.
///
/// Even tho the wrapping seems meaningless, this allows us to provide internal
/// operations that are too difficult or too ugly to implement in the macros that
/// will call the query method of Crud.
#[derive(Debug)]
pub enum CanyonRows {
    #[cfg(feature = "postgres")]
    Postgres(Vec<tokio_postgres::Row>),
    #[cfg(feature = "mssql")]
    Tiberius(Vec<tiberius::Row>),
    #[cfg(feature = "mysql")]
    MySQL(Vec<mysql_async::Row>),
}

impl CanyonRows {
    #[cfg(feature = "postgres")]
    pub fn get_postgres_rows(&self) -> &Vec<tokio_postgres::Row> {
        match self {
            Self::Postgres(v) => v,
            _ => panic!("This branch will never ever should be reachable"),
        }
    }

    #[cfg(feature = "mssql")]
    pub fn get_tiberius_rows(&self) -> &Vec<tiberius::Row> {
        match self {
            Self::Tiberius(v) => v,
            _ => panic!("This branch will never ever should be reachable"),
        }
    }

    #[cfg(feature = "mysql")]
    pub fn get_mysql_rows(&self) -> &Vec<mysql_async::Row> {
        match self {
            Self::MySQL(v) => v,
            _ => panic!("This branch will never ever should be reachable"),
        }
    }

    /// Returns the entity at the given index for the returned rows
    ///
    /// This is just a wrapper get operation over the [Vec] get operation
    pub fn get_row_at(&self, index: usize) -> Option<&dyn Row> {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => v.get(index).map(|inner| inner as &dyn Row),
            #[cfg(feature = "mssql")]
            Self::Tiberius(v) => v.get(index).map(|inner| inner as &dyn Row),
            #[cfg(feature = "mysql")]
            Self::MySQL(v) => v.get(index).map(|inner| inner as &dyn Row),
        }
    }

    pub fn first_row<T: RowMapper<Output = T>>(&self) -> Option<T> {
        let row = match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => v.first().map(|r| T::deserialize_postgresql(r)),
            #[cfg(feature = "mssql")]
            Self::Tiberius(v) => v.first().map(|r| T::deserialize_sqlserver(r)),
            #[cfg(feature = "mysql")]
            Self::MySQL(v) => v.first().map(|r| T::deserialize_mysql(r)),
        };

        row?.ok()
    }

    /// Returns the number of elements present on the wrapped collection
    pub fn len(&self) -> usize {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => v.len(),
            #[cfg(feature = "mssql")]
            Self::Tiberius(v) => v.len(),
            #[cfg(feature = "mysql")]
            Self::MySQL(v) => v.len(),
        }
    }

    /// Returns true whenever the wrapped collection of Rows does not contains any elements
    pub fn is_empty(&self) -> bool {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => v.is_empty(),
            #[cfg(feature = "mssql")]
            Self::Tiberius(v) => v.is_empty(),
            #[cfg(feature = "mysql")]
            Self::MySQL(v) => v.is_empty(),
        }
    }
}

pub trait FromSql<'a>:
    __backend_from_sql::PostgresFromSql<'a>
    + __backend_from_sql::MySqlFromSql
    + __backend_from_sql::MsSqlFromSql<'a>
{
}

impl<'a, T> FromSql<'a> for T where
    T: __backend_from_sql::PostgresFromSql<'a>
        + __backend_from_sql::MySqlFromSql
        + __backend_from_sql::MsSqlFromSql<'a>
{
}

pub trait FromSqlOwnedValue:
    __backend_from_sql_owned::PostgresFromSqlOwned
    + __backend_from_sql_owned::MySqlFromSqlOwned
    + __backend_from_sql_owned::MsSqlFromSqlOwned
{
}

impl<T> FromSqlOwnedValue for T where
    T: __backend_from_sql_owned::PostgresFromSqlOwned
        + __backend_from_sql_owned::MySqlFromSqlOwned
        + __backend_from_sql_owned::MsSqlFromSqlOwned
{
}

#[doc(hidden)]
pub mod __backend_from_sql {
    #[cfg(feature = "postgres")]
    pub trait PostgresFromSql<'a>: tokio_postgres::types::FromSql<'a> {}

    #[cfg(feature = "postgres")]
    impl<'a, T> PostgresFromSql<'a> for T where T: tokio_postgres::types::FromSql<'a> {}

    #[cfg(not(feature = "postgres"))]
    pub trait PostgresFromSql<'a> {}

    #[cfg(not(feature = "postgres"))]
    impl<'a, T> PostgresFromSql<'a> for T {}

    #[cfg(feature = "mysql")]
    pub trait MySqlFromSql: mysql_async::prelude::FromValue {}

    #[cfg(feature = "mysql")]
    impl<T> MySqlFromSql for T where T: mysql_async::prelude::FromValue {}

    #[cfg(not(feature = "mysql"))]
    pub trait MySqlFromSql {}

    #[cfg(not(feature = "mysql"))]
    impl<T> MySqlFromSql for T {}

    #[cfg(feature = "mssql")]
    pub trait MsSqlFromSql<'a>: tiberius::FromSql<'a> {}

    #[cfg(feature = "mssql")]
    impl<'a, T> MsSqlFromSql<'a> for T where T: tiberius::FromSql<'a> {}

    #[cfg(not(feature = "mssql"))]
    pub trait MsSqlFromSql<'a> {}

    #[cfg(not(feature = "mssql"))]
    impl<'a, T> MsSqlFromSql<'a> for T {}
}

#[doc(hidden)]
pub mod __backend_from_sql_owned {
    #[cfg(feature = "postgres")]
    pub trait PostgresFromSqlOwned: tokio_postgres::types::FromSqlOwned {}

    #[cfg(feature = "postgres")]
    impl<T> PostgresFromSqlOwned for T where T: tokio_postgres::types::FromSqlOwned {}

    #[cfg(not(feature = "postgres"))]
    pub trait PostgresFromSqlOwned {}

    #[cfg(not(feature = "postgres"))]
    impl<T> PostgresFromSqlOwned for T {}

    #[cfg(feature = "mysql")]
    pub trait MySqlFromSqlOwned: mysql_async::prelude::FromValue {}

    #[cfg(feature = "mysql")]
    impl<T> MySqlFromSqlOwned for T where T: mysql_async::prelude::FromValue {}

    #[cfg(not(feature = "mysql"))]
    pub trait MySqlFromSqlOwned {}

    #[cfg(not(feature = "mysql"))]
    impl<T> MySqlFromSqlOwned for T {}

    #[cfg(feature = "mssql")]
    pub trait MsSqlFromSqlOwned: tiberius::FromSqlOwned {}

    #[cfg(feature = "mssql")]
    impl<T> MsSqlFromSqlOwned for T where T: tiberius::FromSqlOwned {}

    #[cfg(not(feature = "mssql"))]
    pub trait MsSqlFromSqlOwned {}

    #[cfg(not(feature = "mssql"))]
    impl<T> MsSqlFromSqlOwned for T {}
}
