use crate::{crud::Transaction, mapper::RowMapper, bounds::Row};
use canyon_connection::{canyon_database_connector::DatabaseType, tiberius, tokio_postgres};
use std::{fmt::Debug, marker::PhantomData};

/// Represents a database result after a query, by wrapping the `Vec<Row>` types that comes with the
/// results after the query.
/// and providing methods to deserialize this result into a **user defined struct**
#[derive(Debug)]
pub struct DatabaseResult<T: Debug> {
    pub postgres: Vec<tokio_postgres::Row>,
    pub sqlserver: Vec<tiberius::Row>,
    pub active_ds: DatabaseType,
    _phantom_data: std::marker::PhantomData<T>,
}

impl<T: Debug> DatabaseResult<T> {
    pub fn new_postgresql(result: Vec<tokio_postgres::Row>) -> Self {
        Self {
            postgres: result,
            sqlserver: Vec::with_capacity(0),
            active_ds: DatabaseType::PostgreSql,
            _phantom_data: PhantomData,
        }
    }

    pub fn new_sqlserver(results: Vec<tiberius::Row>) -> Self {
        Self {
            postgres: Vec::with_capacity(0),
            sqlserver: results,
            active_ds: DatabaseType::SqlServer,
            _phantom_data: PhantomData,
        }
    }

    /// Returns a [`Vec<T>`] filled with instances of the type T.
    /// Z param it's used to constrait the types that can call this method.
    ///
    /// Also, provides a way to statically call `Z::deserialize_<db>` method,
    /// which it's a complex implementation used by the macros to automatically
    /// map database columns into the fields for T.
    pub fn get_entities<Z: RowMapper<T> + Debug>(&self) -> Vec<T>
    where
        T: Transaction<T>,
    {
        match self.active_ds {
            DatabaseType::PostgreSql => self.map_from_postgresql::<Z>(),
            DatabaseType::SqlServer => self.map_from_sql_server::<Z>(),
        }
    }

    fn map_from_postgresql<Z: RowMapper<T> + Debug>(&self) -> Vec<T>
    where
        T: Transaction<T>,
    {
        let mut results = Vec::new();

        self.postgres
            .iter()
            .for_each(|row| results.push(Z::deserialize_postgresql(row)));

        results
    }

    fn map_from_sql_server<Z: RowMapper<T> + Debug>(&self) -> Vec<T>
    where
        T: Transaction<T>,
    {
        let mut results = Vec::new();

        self.sqlserver
            .iter()
            .for_each(|row| results.push(Z::deserialize_sqlserver(row)));

        results
    }

    pub fn as_canyon_row(&self) -> Vec<&dyn Row> {
        let mut results = Vec::new();

        match self.active_ds {
            DatabaseType::PostgreSql => {
                self.postgres
                    .iter()
                    .for_each(|row| results.push(
                        row as &dyn Row
                    ));
            },
            DatabaseType::SqlServer => {
                self.sqlserver
                    .iter()
                    .for_each(|row| results.push(
                        row as &dyn Row
                    ));
            },
        };

        results
    }

    /// Returns the active datasource
    pub fn get_active_ds(&self) -> &DatabaseType {
        &self.active_ds
    }

    /// Returns how many rows contains the result of the query
    pub fn number_of_results(&self) -> usize {
        match self.active_ds {
            DatabaseType::PostgreSql => self.postgres.len(),
            DatabaseType::SqlServer => self.sqlserver.len(),
        }
    }
}
