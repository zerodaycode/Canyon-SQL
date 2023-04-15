use tokio_postgres::types::FromSql;
use crate::bounds::{PrimaryKey, QueryParameter};
use crate::crud::Transaction;
use crate::mapper::RowMapper;

/// Lightweight wrapper over the collection of results of the different crates
/// supported by Canyon-SQL.
///
/// Even tho the wrapping seems meaningless, this allows us to provide internal
/// operations that are too difficult or to ugly to implement in the macros that
/// will call the query method of Crud.
pub enum CanyonRows {
    #[cfg(feature = "postgres")] Postgres(Vec<tokio_postgres::Row>),
    #[cfg(feature = "mssql")] Tiberius(Vec<Vec<tiberius::Row>>)
}

impl CanyonRows {
    // /// Type constructor, returning the correct variant of Self wrapping the collection of results
    // /// by the given database connection
    // pub fn new(
    //     conn: &DatabaseConnection,
    //     res: Vec<T>
    // ) -> Self {
    //     match conn {
    //         #[cfg(feature = "postgres")] DatabaseConnection::Postgres(_) => Self::Postgres(res),
    //         #[cfg(feature = "mssql")] DatabaseConnection::SqlServer(_) => Self::Tiberius(res)
    //     }
    // }

    /// Consumes `self` and returns the wrapped [`std::vec::Vec`] with the instances of T
    pub fn into_results<T>(self) -> Vec<T> where T: Transaction<T> + RowMapper<T> {
        match self {
            #[cfg(feature = "postgres")] Self::Postgres(v) => v
                .iter()
                .map(|row| T::deserialize_postgresql(row))
                .collect(),
            #[cfg(feature = "mssql")] Self::Tiberius(v) => v
                .iter()
                .flatten()
                .map(|row| T::deserialize_sqlserver(&row))
                .collect()
        }
    }

    ///
    pub fn set_primary_key_after_insert<'a, T, PkType: PrimaryKey>(self, pk: &str) -> PkType {
        match self {
            #[cfg(feature = "postgres")] Self::Postgres(v) => {
                v.get(0)
                    .expect("No value found on the returning clause")
                    .get::<&str, PkType>(pk)
                    // .to_owned();
            }
            #[cfg(feature = "mssql")] Self::Tiberius(v) => {
                v.into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .remove(0)
                    .get::<PkType, &str>(pk)
                    .expect("SQL Server primary key type failed to be set as value")
                    // .to_owned()
            }
        }
    }
}

// r.iter().map(|row| T::deserialize_postgresql(row)).collect()
// .map(|row| T::deserialize_sqlserver(&row))


// canyon_sql::crud::DatabaseType::SqlServer => {
// self.#pk_ident = res.sqlserver.get(0)
// .expect("No value found on the returning clause")
// .get::<#pk_type, &str>(#primary_key)
// .expect("SQL Server primary key type failed to be set as value")
// .to_owned();
