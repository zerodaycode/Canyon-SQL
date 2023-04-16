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
    #[cfg(feature = "tiberius")] Tiberius(Vec<Vec<tiberius::Row>>),
    UnusableTypeMarker(PhantomData<T>)
}

impl<T> CanyonRows<T> {
    // /// Type constructor, returning the correct variant of Self wrapping the collection of results
    // /// by the given database connection
    // pub fn new(
    //     conn: &DatabaseConnection,
    //     res: Vec<T>
    // ) -> Self {
    //     match conn {
    //         #[cfg(feature = "tokio-postgres")] DatabaseConnection::Postgres(_) => Self::Postgres(res),
    //         #[cfg(feature = "tiberius")] DatabaseConnection::SqlServer(_) => Self::Tiberius(res)
    //     }
    // }

    /// Consumes `self` and returns the wrapped [`std::vec::Vec`] with the instances of T
    pub fn into_results<Z: RowMapper<T>>(self) -> Vec<T> where T: Transaction<T> {
        match self {
            #[cfg(feature = "tokio-postgres")] Self::Postgres(v) => v
                .iter()
                .map(|row| Z::deserialize_postgresql(row))
                .collect(),
            #[cfg(feature = "tiberius")] Self::Tiberius(v) => v
                .iter()
                .flatten()
                .map(|row| Z::deserialize_sqlserver(&row))
                .collect(),
            _ => panic!("This branch will never ever should be reachable")
        }
    }
}
