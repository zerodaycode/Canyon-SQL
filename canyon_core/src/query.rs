use std::fmt::Display;

use async_trait::async_trait;

use crate::{query_parameters::QueryParameter, rows::CanyonRows};

#[async_trait]
pub trait DbConnection {
    async fn launch(
        &self,
        stmt: &str,
            params: &[&dyn QueryParameter<'_>],
    ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send + 'static)>>;
}

#[async_trait]
pub trait Transaction<T> {
    // provisional name
    /// Performs a query against the targeted database by the selected or
    /// the defaulted datasource, wrapping the resultant collection of entities
    /// in [`super::rows::CanyonRows`]
    async fn query<'a, C, S, Z>(
        // &self,
        stmt: S,
        params: Z,
        db_conn: &C,
    ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send + 'static)>>
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a,
        C: DbConnection + Display + Sync + Send + 'a,
    {
        // Ok(CanyonRows::Postgres(vec![]))
        todo!()
    }
}
