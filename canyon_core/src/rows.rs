#[cfg(feature = "mysql")]
use mysql_async::{self};
#[cfg(feature = "mssql")]
use tiberius::{self};
#[cfg(feature = "postgres")]
use tokio_postgres::{self};

use crate::mapper::{CanyonError, IntoResults, RowMapper};
use crate::row::Row;

use cfg_if::cfg_if;

// Helper macro to conditionally add trait bounds
// these are the hacky intermediate traits
cfg_if! {
    if #[cfg(all(feature = "postgres", feature = "mysql", feature = "mssql"))] {
      pub trait FromSql<'a, T>: tokio_postgres::types::FromSql<'a>
        + tiberius::FromSql<'a>
        + mysql_async::prelude::FromValue {}
      impl<'a, T> FromSql<'a, T> for T where T:
        tokio_postgres::types::FromSql<'a>
        + tiberius::FromSql<'a>
        + mysql_async::prelude::FromValue
        {}

      pub trait FromSqlOwnedValue<T>: tokio_postgres::types::FromSqlOwned
        + tiberius::FromSqlOwned
        + mysql_async::prelude::FromValue {}
      impl<T> FromSqlOwnedValue<T> for T where T:
        tokio_postgres::types::FromSqlOwned
        + tiberius::FromSqlOwned
        + mysql_async::prelude::FromValue
        {}
    } else if #[cfg(feature = "postgres")] {
      pub trait FromSql<'a, T>: tokio_postgres::types::FromSql<'a> {}
      impl<'a, T> FromSql<'a, T> for T where T:
        tokio_postgres::types::FromSql<'a> {}

      pub trait FromSqlOwnedValue<T>: tokio_postgres::types::FromSqlOwned {}
      impl<T> FromSqlOwnedValue<T> for T where T:
        tokio_postgres::types::FromSqlOwned {}
    } else if #[cfg(feature = "mssql")] {
      pub trait FromSql<'a, T>: tiberius::FromSqlOwned {}
      impl<'a, T> FromSql<'a, T> for T where T: tiberius::FromSqlOwned {}

      pub trait FromSqlOwnedValue<T>: tiberius::FromSqlOwned {}
      impl<T> FromSqlOwnedValue<T> for T where T: tiberius::FromSqlOwned {}
    }
    // TODO: missing combinations else
}

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

impl IntoResults for Result<CanyonRows, CanyonError> {
    fn into_results<R>(self) -> Result<Vec<R>, CanyonError>
    where
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        self.map(move |rows| rows.into_results::<R>())
    }
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

    /// Consumes `self` and returns the wrapped [`std::vec::Vec`] with the instances of R
    pub fn into_results<R>(self) -> Vec<R>
    where
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => v.iter().map(|row| R::deserialize_postgresql(row)).collect(),
            #[cfg(feature = "mssql")]
            Self::Tiberius(v) => v.iter().map(|row| R::deserialize_sqlserver(row)).collect(),
            #[cfg(feature = "mysql")]
            Self::MySQL(v) => v.iter().map(|row| R::deserialize_mysql(row)).collect(),
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
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => v.first().map(|r| T::deserialize_postgresql(r)),
            #[cfg(feature = "mssql")]
            Self::Tiberius(v) => v.first().map(|r| T::deserialize_sqlserver(r)),
            #[cfg(feature = "mysql")]
            Self::MySQL(v) => v.first().map(|r| T::deserialize_mysql(r)),
        }
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
