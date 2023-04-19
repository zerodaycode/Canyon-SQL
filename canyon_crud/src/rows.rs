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

    /// Consumes `self` and returns the wrapped [`std::vec::Vec`] with the instances of T
    pub fn into_results<Z: RowMapper<T>>(self) -> Vec<T>
    where
        T: Transaction<T>,
    {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => v.iter().map(|row| Z::deserialize_postgresql(row)).collect(),
            #[cfg(feature = "mssql")]
            Self::Tiberius(v) => v.iter().map(|row| Z::deserialize_sqlserver(&row)).collect(),
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
            _ => panic!("This branch will never ever should be reachable"),
        }
    }
}

// #[cfg(feature = "postgres")]
// impl<T> IntoIterator for CanyonRows<T> {
//     type Item = tokio_postgres::Row;
//     type IntoIter = std::vec::IntoIter<Self::Item>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         match self {
//             Self::Postgres(v) => v.into_iter(),
//             _ => panic!()
//         }
//     }
// }
//
// #[cfg(feature = "mssql")]
// impl<T> IntoIterator for CanyonRows<T> {
//     type Item = tiberius::Row;
//     type IntoIter = std::vec::IntoIter<Self::Item>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         match self {
//             Self::Tiberius(v) => v.into_iter(),
//             _ => panic!()
//         }
//     }
// }
//
// #[cfg(all(feature = "tokio-postgres", feature = "tiberius"))]
// impl<T> IntoIterator for CanyonRows<T> {
//     if cfg!(feature = "tokio-postgres") {
//     type Item = tokio_postgres::Row;
//     } else { type Item = tiberius::Row; }
//     type IntoIter = std::vec::IntoIter<Self::Item>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         match self {
//             Self::Tiberius(v) => v.into_iter(),
//             _ => panic!()
//         }
//     }
// }
