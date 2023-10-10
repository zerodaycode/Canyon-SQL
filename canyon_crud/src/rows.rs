use crate::crud::Transaction;
use crate::mapper::RowMapper;
use std::marker::PhantomData;

/// Lightweight wrapper over the collection of results of the different crates
/// supported by Canyon-SQL.
///
/// Even tho the wrapping seems meaningless, this allows us to provide internal
/// operations that are too difficult or to ugly to implement in the macros that
/// will call the query method of Crud.
pub enum CanyonRows<T> {
    #[cfg(feature = "postgres")]
    Postgres(Vec<tokio_postgres::Row>),
    #[cfg(feature = "mssql")]
    Tiberius(Vec<tiberius::Row>),
    #[cfg(feature = "mysql")]
    MySQL(Vec<mysql_async::Row>),

    UnusableTypeMarker(PhantomData<T>),
}

impl<T> CanyonRows<T> {
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
    pub fn into_results<Z: RowMapper<T>>(self) -> Vec<T>
    where
        T: Transaction<T>,
    {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => v.iter().map(|row| Z::deserialize_postgresql(row)).collect(),
            #[cfg(feature = "mssql")]
            Self::Tiberius(v) => v.iter().map(|row| Z::deserialize_sqlserver(row)).collect(),
            #[cfg(feature = "mysql")]
            Self::MySQL(v) => v.iter().map(|row| Z::deserialize_mysql(row)).collect(),
            _ => panic!("This branch will never ever should be reachable"),
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
            _ => panic!("This branch will never ever should be reachable"),
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
            _ => panic!("This branch will never ever should be reachable"),
        }
    }
}
