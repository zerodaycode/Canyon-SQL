use std::fmt::Display;

use async_trait::async_trait;

use crate::{query_parameters::QueryParameter, rows::CanyonRows};

#[async_trait]
pub trait DbConnection {
    async fn launch<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send + 'static)>>;
}

#[async_trait]
pub trait Transaction<T> {
    // provisional name
    /// Performs a query against the targeted database by the selected or
    /// the defaulted datasource, wrapping the resultant collection of entities
    /// in [`super::rows::CanyonRows`]
    async fn query<'a, C, S, Z>(
        stmt: S,
        params: Z,
        db_conn: &mut C,
    ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send + 'static)>>
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a,
        C: DbConnection + Sync + Send + 'a,
    {
        db_conn.launch(stmt.as_ref(), params.as_ref()).await
    }
}
