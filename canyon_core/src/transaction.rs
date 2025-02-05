use crate::connection::db_connector::DbConnection;
use crate::{query_parameters::QueryParameter, rows::CanyonRows};
use std::error::Error;
use std::{fmt::Display, future::Future};
use crate::mapper::RowMapper;
use crate::row::Row;

pub trait Transaction<T> {
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
        async move { input.query(stmt.as_ref(), params.as_ref()).await }
    }

    /// 
    /// *Impl notes:* allow async fn in trait is provisionally here because we have to
    /// rework certain details around the bounds of QueryParameter
    #[allow(async_fn_in_trait)]
    async fn query_rows<'a, S, Z, R: RowMapper<R>>(
        stmt: S,
        params: Z,
        input: impl DbConnection + Send + 'a,
    ) -> Result<Vec<R>, Box<(dyn Error + Sync + Send + 'a)>>
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a,
    {
        input.query_rows(stmt, params).await
    }

    fn query_one<'a, S, Z, R: RowMapper<R>>(
        stmt: S,
        params: Z,
        input: impl DbConnection + Send + 'a,
    ) -> impl Future<Output = Result<Option<R>, Box<(dyn Error + Sync + Send)>>> + Send
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a,
    {
        async move { input.query_one(stmt.as_ref(), params.as_ref()).await }
    }
}
