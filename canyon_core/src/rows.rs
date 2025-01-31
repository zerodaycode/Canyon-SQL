#[cfg(feature = "mysql")]
use mysql_async::{self};
#[cfg(feature = "mssql")]
use tiberius::{self};
#[cfg(feature = "postgres")]
use tokio_postgres::{self};

use crate::mapper::{CanyonError, IntoResults, RowMapper};
use crate::row::Row;
use std::error::Error;

use cfg_if::cfg_if;

// Helper macro to conditionally add trait bounds
// these are the hacky intermediate traits
cfg_if! {
  // if #[cfg(feature = "postgres")] {
  //   trait FromSql<'a, T> where T: tokio_postgres::types::FromSql<'a> { }
  // } else if #[cfg(feature = "mssql")] {
  //   trait FromSql<'a, T> where T: tiberius::FromSql<'a> { }
  // } else if #[cfg(feature = "mysql")] {
  //   trait FromSql<'a, T> where T: mysql_async::types::FromSql<'a> { }
  // }
    if #[cfg(feature = "postgres")]  {
      pub trait FromSql<'a, T>: tokio_postgres::types::FromSql<'a>
        + tiberius::FromSql<'a>
        + mysql_async::prelude::FromValue {}
      impl<'a, T> FromSql<'a, T> for T where T:
        tokio_postgres::types::FromSql<'a>
        + tiberius::FromSql<'a>
        + mysql_async::prelude::FromValue
        {}
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
    fn into_results<T>(self) -> Result<Vec<T>, CanyonError>
    where
        T: RowMapper<T>,
    {
        self.map(move |rows| rows.into_results::<T>())
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

    /// Consumes `self` and returns the wrapped [`std::vec::Vec`] with the instances of T
    pub fn into_results<Z: RowMapper<Z>>(self) -> Vec<Z> {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => v.iter().map(|row| Z::deserialize_postgresql(row)).collect(),
            #[cfg(feature = "mssql")]
            Self::Tiberius(v) => v.iter().map(|row| Z::deserialize_sqlserver(row)).collect(),
            #[cfg(feature = "mysql")]
            Self::MySQL(v) => v.iter().map(|row| Z::deserialize_mysql(row)).collect(),
        }
    }

    /// Returns the entity at the given index for the returned rows
    ///
    /// This is just a wrapper get operation over the [Vec::get] operation
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

    pub fn get_column_at_row<'a, C: FromSql<'a, C>>(
        &'a self,
        column_name: &str,
        index: usize,
    ) -> Result<C, Box<dyn Error + Send + Sync>> {
        let row_extraction_failure = || {
            format!(
                "{:?} - Failure getting the row: {} at index: {}",
                self, column_name, index
            )
        };

        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => Ok(v
                .get(index)
                .ok_or_else(row_extraction_failure)?
                .get::<&str, C>(column_name)),
            #[cfg(feature = "mssql")]
            Self::Tiberius(ref v) => v
                .get(index)
                .ok_or_else(row_extraction_failure)?
                .get::<C, &str>(column_name)
                .ok_or_else(|| {
                    format!(
                        "{:?} - Failed to obtain the RETURNING value for an insert operation",
                        self
                    )
                    .into()
                }),
            #[cfg(feature = "mysql")]
            Self::MySQL(ref v) => v
                .get(index)
                .ok_or_else(row_extraction_failure)?
                .get::<C, usize>(0)
                .ok_or_else(|| {
                    format!(
                        "{:?} - Failed to obtain the RETURNING value for an insert operation",
                        self
                    )
                    .into()
                }),
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
