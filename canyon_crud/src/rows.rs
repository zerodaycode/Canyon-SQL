use std::marker::PhantomData;
use crate::crud::Transaction;
use crate::mapper::RowMapper;

/// Lightweight wrapper over the collection of results of the different crates
/// supported by Canyon-SQL.
///
/// Even tho the wrapping seems meaningless, this allows us to provide internal
/// operations that are too difficult or to ugly to implement in the macros that
/// will call the query method of Crud.
pub enum CanyonRows<T> {
    #[cfg(feature = "tokio-postgres")] Postgres(Vec<tokio_postgres::Row>),
    #[cfg(feature = "tiberius")] Tiberius(Vec<tiberius::Row>),
    UnusableTypeMarker(PhantomData<T>)
}

impl<T> CanyonRows<T> {
    #[cfg(feature = "tokio-postgres")]
    pub fn get_postgres_rows(self) -> Vec<tokio_postgres::Row> {
        match self {
            Self::Postgres(v) => v,
            _ => panic!("This branch will never ever should be reachable")
        }
    }

    #[cfg(feature = "tiberius")]
    pub fn get_tiberius_rows(self) -> Vec<tiberius::Row> {
        match self {
            Self::Tiberius(v) => v,
            _ => panic!("This branch will never ever should be reachable")
        }
    }

    /// Consumes `self` and returns the wrapped [`std::vec::Vec`] with the instances of T
    pub fn into_results<Z: RowMapper<T>>(self) -> Vec<T> where T: Transaction<T> {
        match self {
            #[cfg(feature = "tokio-postgres")] Self::Postgres(v) => v
                .iter()
                .map(|row| Z::deserialize_postgresql(row))
                .collect(),
            #[cfg(feature = "tiberius")] Self::Tiberius(v) => v
                .iter()
                .map(|row| Z::deserialize_sqlserver(&row))
                .collect(),
            _ => panic!("This branch will never ever should be reachable")
        }
    }
}

#[cfg(feature = "tokio-postgres")]
impl<T> IntoIterator for CanyonRows<T> {
    type Item = tokio_postgres::Row;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Postgres(v) => v.into_iter(),
            _ => panic!()
        }
    }
}

#[cfg(feature = "tiberius")]
impl<T> IntoIterator for CanyonRows<T> {
    type Item = tiberius::Row;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Tiberius(v) => v.into_iter(),
            _ => panic!()
        }
    }
}
