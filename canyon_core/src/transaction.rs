use crate::connection::db_connector::DbConnection;
use crate::mapper::RowMapper;
use crate::rows::FromSqlOwnedValue;
use crate::{query_parameters::QueryParameter, rows::CanyonRows};
use std::error::Error;
use std::{fmt::Display, future::Future};

pub trait Transaction<T> {
    fn query<'a, S, R: RowMapper<R>>(
        stmt: S,
        params: &[&'a (dyn QueryParameter<'a>)],
        input: impl DbConnection + Send,
    ) -> impl Future<Output = Result<Vec<R>, Box<(dyn Error + Sync + Send)>>>
    where
        S: AsRef<str> + Display + Send,
    {
        async move { input.query(stmt, params).await }
    }

    fn query_one<'a, S, Z, R: RowMapper<R>>(
        stmt: S,
        params: Z,
        input: impl DbConnection + Send + 'a,
    ) -> impl Future<Output = Result<Option<R>, Box<(dyn Error + Sync + Send)>>> + Send
    where
        S: AsRef<str> + Display + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Send + 'a,
    {
        async move { input.query_one(stmt.as_ref(), params.as_ref()).await }
    }

    fn query_one_for<'a, S, Z, F: FromSqlOwnedValue<F>>(
        stmt: S,
        params: Z,
        input: impl DbConnection + Send + 'a,
    ) -> impl Future<Output = Result<F, Box<(dyn Error + Sync + Send)>>> + Send
    where
        S: AsRef<str> + Display + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Send + 'a,
    {
        async move { input.query_one_for(stmt.as_ref(), params.as_ref()).await }
    }

    /// Performs a query against the targeted database by the selected or
    /// the defaulted datasource, wrapping the resultant collection of entities
    /// in [`super::rows::CanyonRows`]
    fn query_rows<'a, S, Z>(
        stmt: S,
        params: Z,
        input: impl DbConnection + Send + 'a,
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn Error + Sync + Send)>>> + Send
    where
        S: AsRef<str> + Display + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Send + 'a,
    {
        async move { input.query_rows(stmt.as_ref(), params.as_ref()).await }
    }
}
