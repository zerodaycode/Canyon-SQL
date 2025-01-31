use crate::connection::db_connector::DbConnection;
use crate::{query_parameters::QueryParameter, rows::CanyonRows};
use std::error::Error;
use std::{fmt::Display, future::Future};

pub trait Transaction<T> {
    // provisional name
    /// Performs a query against the targeted database by the selected or
    /// the defaulted datasource, wrapping the resultant collection of entities
    /// in [`super::rows::CanyonRows`]
    fn query<'a, S, Z>(
        stmt: S,
        params: Z,
        input: impl DbConnection + Send + 'a,
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn Error + Sync + Send)>>> + Send
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a,
    {
        async move { input.launch(stmt.as_ref(), params.as_ref()).await }
    }
}
