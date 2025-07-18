use crate::connection::clients::postgresql::PostgreSqlConnection;
use crate::{
    connection::{
        clients::postgresql::postgres_query_launcher, contracts::DbConnection,
        database_type::DatabaseType,
    },
    mapper::RowMapper,
    query::parameters::QueryParameter,
    rows::{CanyonRows, FromSqlOwnedValue},
};
use std::{error::Error, future::Future};

impl DbConnection for PostgreSqlConnection {
    fn query_rows(
        &self,
        stmt: &str,
        params: &[&dyn QueryParameter],
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn Error + Send + Sync)>>> + Send {
        postgres_query_launcher::query_rows(stmt, params, self)
    }

    fn query<S, R>(
        &self,
        stmt: S,
        params: &[&(dyn QueryParameter)],
    ) -> impl Future<Output = Result<Vec<R>, Box<(dyn Error + Send + Sync)>>> + Send
    where
        S: AsRef<str> + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        postgres_query_launcher::query(stmt, params, self)
    }

    fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&(dyn QueryParameter)],
    ) -> impl Future<Output = Result<Option<R::Output>, Box<(dyn Error + Send + Sync)>>> + Send
    where
        R: RowMapper,
    {
        postgres_query_launcher::query_one::<R>(stmt, params, self)
    }

    fn query_one_for<T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&(dyn QueryParameter)],
    ) -> impl Future<Output = Result<T, Box<(dyn Error + Send + Sync)>>> + Send {
        postgres_query_launcher::query_one_for(stmt, params, self)
    }

    fn execute(
        &self,
        stmt: &str,
        params: &[&(dyn QueryParameter)],
    ) -> impl Future<Output = Result<u64, Box<(dyn Error + Send + Sync)>>> + Send {
        postgres_query_launcher::execute(stmt, params, self)
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Send + Sync)>> {
        Ok(DatabaseType::PostgreSql)
    }
}
