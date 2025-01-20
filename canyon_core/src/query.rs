use std::fmt::Display;

use async_trait::async_trait;

use crate::{query_parameters::QueryParameter, rows::CanyonRows};

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
    ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send + 'static)>>
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a,
        C: DbConnection + Sync + Send + 'a,
        I: Into<TransactionInput<C>> + Sync + Send + 'a
    {
        let transaction_input= input.into();
        match transaction_input {
            TransactionInput::DbConnection(mut conn) => {
                conn.launch(stmt.as_ref(), params.as_ref()).await
            }
            // TransactionInput::DatasourceConfig(ds) => {
            //     let mut conn = DatabaseConnection::new(&ds).await?;
            //     conn.launch(stmt.as_ref(), params).await
            // }
            TransactionInput::DatasourceName(ds_name) => {
                let ds = get_database_connection_by_ds(Some(ds_name))?;
                let mut conn = DatabaseConnection::new(ds).await?;
                conn.launch(stmt.as_ref(), params).await
            }
        }
    }
}

pub enum TransactionInput<T: DbConnection> {
    DbConnection(T),
    // DatasourceConfig(DatasourceConfig),
    DatasourceName(String),
}

impl<T: DbConnection> From<T> for TransactionInput<T> {
    fn from(conn: T) -> Self {
        TransactionInput::DbConnection(conn)
    }
}

// impl<T: DbConnection> From<DatasourceConfig> for TransactionInput {
//     fn from(ds: DatasourceConfig) -> Self {
//         TransactionInput::DatasourceConfig(ds)
//     }
// }

impl<T: DbConnection> From<String> for TransactionInput<T> {
    fn from(ds_name: String) -> Self {
        TransactionInput::DatasourceName(ds_name)
    }
}

impl<T: DbConnection> From<&str> for TransactionInput<T> {
    fn from(ds_name: &str) -> Self {
        TransactionInput::DatasourceName(ds_name.to_string())
    }
}
