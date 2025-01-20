use std::fmt::Display;

use async_trait::async_trait;

use crate::{connection::{datasources::DatasourceConfig, db_connector::DatabaseConnection, get_database_connection_by_ds}, query_parameters::{self, QueryParameter}, rows::CanyonRows};

// TODO: in order to avoid the tiberius transmute, we should define other method that takes the db_conn as a mut ref

#[async_trait]
pub trait DbConnection {
    // TODO: guess that this is the trait that must remain sealed
    async fn launch<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send + 'static)>>;
}

pub trait Datasource: DbConnection{}

#[async_trait]
pub trait Transaction<T> {
    // provisional name
    /// Performs a query against the targeted database by the selected or
    /// the defaulted datasource, wrapping the resultant collection of entities
    /// in [`super::rows::CanyonRows`]
    async fn query<'a, C, S, Z, I>(
        stmt: S,
        params: Z,
        input: I,
    ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send + 'a)>>
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a,
        C: DbConnection + Sync + Send + 'a,
        I: Into<TransactionInput<'a, C>> + Sync + Send + 'a
    {
        let transaction_input= input.into();
        let statement = stmt.as_ref();
        let query_parameters = params.as_ref();

        match transaction_input {
            TransactionInput::DbConnection(conn) => {
                conn.launch(statement, query_parameters).await
            }
            TransactionInput::DbConnectionRef(conn) => {
                conn.launch(statement, query_parameters).await
            }
            TransactionInput::DbConnectionRefMut(/* TODO: mut*/ conn) => {
                conn.launch(statement, query_parameters).await
            }
            TransactionInput::DatasourceConfig(ds) => { // TODO: add a new from_ds_config_mut for mssql
                let conn = DatabaseConnection::new(&ds).await?;
                conn.launch(statement, query_parameters).await
            }
            TransactionInput::DatasourceName(ds_name) => {
                let conn = get_database_connection_by_ds(Some(ds_name)).await?;
                conn.launch(statement, query_parameters).await
            }
        }
    }
}

pub enum TransactionInput<'a, T: DbConnection> {
    DbConnection(T),
    DbConnectionRef(&'a T),
    DbConnectionRefMut(&'a mut T),
    DatasourceConfig(&'a DatasourceConfig),
    DatasourceName(&'a str),
}

impl<'a, T: DbConnection> From<T> for TransactionInput<'a, T> {
    fn from(conn: T) -> Self {
        TransactionInput::DbConnection(conn)
    }
}

impl<'a, T: DbConnection> From<&'a T> for TransactionInput<'a, T> {
    fn from(conn: &'a T) -> Self {
        TransactionInput::DbConnectionRef(conn)
    }
}

impl<'a, T: DbConnection> From<&'a mut T> for TransactionInput<'a, T> {
    fn from(conn: &'a mut T) -> Self {
        TransactionInput::DbConnectionRefMut(conn)
    }
}

impl<'a, T: DbConnection> From<&'a DatasourceConfig> for TransactionInput<'a, T> {
    fn from(ds: &'a DatasourceConfig) -> Self {
        TransactionInput::DatasourceConfig(ds)
    }
}

impl<'a, T: DbConnection> From<&'a str> for TransactionInput<'a, T> {
    fn from(ds_name: &'a str) -> Self {
        TransactionInput::DatasourceName(ds_name)
    }
}
