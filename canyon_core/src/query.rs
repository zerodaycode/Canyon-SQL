use crate::{
    connection::{
        datasources::DatasourceConfig, db_connector::DatabaseConnection,
        get_database_connection_by_ds,
    },
    query_parameters::QueryParameter,
    rows::CanyonRows,
};
use std::{fmt::Display, future::Future};

// TODO: in order to avoid the tiberius transmute, we should define other method that takes the db_conn as a mut ref

pub trait DbConnection {
    // TODO: guess that this is the trait that must remain sealed
    fn launch<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send)>>> + Send;
}

pub trait Transaction<T> {
    // provisional name
    /// Performs a query against the targeted database by the selected or
    /// the defaulted datasource, wrapping the resultant collection of entities
    /// in [`super::rows::CanyonRows`]
    fn query<'a, S, Z, I>(
        stmt: S,
        params: Z,
        input: I,
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send)>>> + Send
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a,
        I: Into<TransactionInput<'a>> + Sync + Send + 'a,
    {
        async move {
            let transaction_input = input.into();
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
                TransactionInput::DatasourceConfig(ds) => {
                    // TODO: add a new from_ds_config_mut for mssql
                    let conn = DatabaseConnection::new(ds).await?;
                    conn.launch(statement, query_parameters).await
                }
                TransactionInput::DatasourceName(ds_name) => {
                    let sane_ds_name = if !ds_name.is_empty() {
                        Some(ds_name)
                    } else {
                        None
                    };
                    let conn = get_database_connection_by_ds(sane_ds_name).await?;
                    conn.launch(statement, query_parameters).await
                }
            }
        }
    }
}

pub enum TransactionInput<'a> {
    DbConnection(DatabaseConnection),
    DbConnectionRef(&'a DatabaseConnection),
    DbConnectionRefMut(&'a mut DatabaseConnection),
    DatasourceConfig(&'a DatasourceConfig),
    DatasourceName(&'a str),
}

impl From<DatabaseConnection> for TransactionInput<'_> {
    fn from(conn: DatabaseConnection) -> Self {
        TransactionInput::DbConnection(conn)
    }
}

impl<'a> From<&'a DatabaseConnection> for TransactionInput<'a> {
    fn from(conn: &'a DatabaseConnection) -> Self {
        TransactionInput::DbConnectionRef(conn)
    }
}

impl<'a> From<&'a mut DatabaseConnection> for TransactionInput<'a> {
    fn from(conn: &'a mut DatabaseConnection) -> Self {
        TransactionInput::DbConnectionRefMut(conn)
    }
}

impl<'a> From<&'a DatasourceConfig> for TransactionInput<'a> {
    fn from(ds: &'a DatasourceConfig) -> Self {
        TransactionInput::DatasourceConfig(ds)
    }
}

impl<'a> From<&'a str> for TransactionInput<'a> {
    fn from(ds_name: &'a str) -> Self {
        TransactionInput::DatasourceName(ds_name)
    }
}
