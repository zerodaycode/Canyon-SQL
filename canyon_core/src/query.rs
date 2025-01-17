use std::fmt::Display;

use async_trait::async_trait;

use crate::{query_parameters::QueryParameter, rows::CanyonRows};

pub trait DbConnection{}

#[async_trait]
pub trait Transaction<T> { // provisional name
    /// Performs a query against the targeted database by the selected or
    /// the defaulted datasource, wrapping the resultant collection of entities
    /// in [`super::rows::CanyonRows`]
    async fn query<'a, S, Z>(
        stmt: S,
        params: Z,
        database_conn: impl DatabaseConnection + Send
    ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send + 'static)>>
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a {
            Ok(CanyonRows::Postgres(vec![]))
        }
}
