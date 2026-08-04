use canyon_core::{
    connection::{contracts::DbConnection, database_type::DatabaseType},
    mapper::RowMapper,
    query::{
        parameters::QueryParameter,
        querybuilder::{DeleteQueryBuilder, SelectQueryBuilder, UpdateQueryBuilder},
    },
};
use std::{error::Error, future::Future};

pub trait ReadOperations<R>: Send
where
    R: RowMapper,
    Vec<R>: FromIterator<R::Output>,
{
    fn find_all() -> impl Future<Output = Result<Vec<R>, Box<dyn Error + Send + Sync>>> + Send;

    fn find_all_with<'connection, I>(
        input: I,
    ) -> impl Future<Output = Result<Vec<R>, Box<dyn Error + Send + Sync>>> + Send
    where
        I: DbConnection + Send + 'connection;

    fn select_query<'a>() -> Result<SelectQueryBuilder<'a>, Box<dyn Error + Send + Sync + 'a>>;

    fn select_query_with<'a>(
        database_type: DatabaseType,
    ) -> Result<SelectQueryBuilder<'a>, Box<dyn Error + Send + Sync + 'a>>;

    fn count() -> impl Future<Output = Result<i64, Box<dyn Error + Send + Sync>>> + Send;

    fn count_with<'connection, I>(
        input: I,
    ) -> impl Future<Output = Result<i64, Box<dyn Error + Send + Sync + 'connection>>> + Send
    where
        I: DbConnection + Send + 'connection;

    fn find_by_pk<'value, 'error>(
        value: &'value dyn QueryParameter,
    ) -> impl Future<Output = Result<Option<R>, Box<dyn Error + Send + Sync + 'error>>> + Send;

    fn find_by_pk_with<'value, 'error, I>(
        value: &'value dyn QueryParameter,
        input: I,
    ) -> impl Future<Output = Result<Option<R>, Box<dyn Error + Send + Sync + 'error>>> + Send
    where
        I: DbConnection + Send + 'value;
}

pub trait InsertOperations: Send {
    fn insert<'entity, 'error>(
        &'entity mut self,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'error>>> + Send;

    fn insert_with<'connection, I>(
        &mut self,
        input: I,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'connection>>> + Send
    where
        I: DbConnection + Send + 'connection;
}

pub trait UpdateOperations: Send {
    fn update(&self) -> impl Future<Output = Result<u64, Box<dyn Error + Send + Sync>>> + Send;

    fn update_with<'connection, I>(
        &self,
        input: I,
    ) -> impl Future<Output = Result<u64, Box<dyn Error + Send + Sync + 'connection>>> + Send
    where
        I: DbConnection + Send + 'connection;

    fn update_query<'a>() -> UpdateQueryBuilder<'a>;

    fn update_query_with<'a>(database_type: DatabaseType) -> UpdateQueryBuilder<'a>;
}

pub trait DeleteOperations: Send {
    fn delete(&self) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send;

    fn delete_with<'connection, 'error, I>(
        &self,
        input: I,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'error>>> + Send
    where
        I: DbConnection + Send + 'connection;

    fn delete_query<'a, 'b>() -> DeleteQueryBuilder<'a>
    where
        'a: 'b;

    fn delete_query_with<'a, 'b>(database_type: DatabaseType) -> DeleteQueryBuilder<'a>
    where
        'a: 'b;
}

pub trait CrudOperations<R>:
    ReadOperations<R> + InsertOperations + UpdateOperations + DeleteOperations
where
    R: RowMapper,
    Vec<R>: FromIterator<R::Output>,
{
}

impl<T, R> CrudOperations<R> for T
where
    T: ReadOperations<R> + InsertOperations + UpdateOperations + DeleteOperations,
    R: RowMapper,
    Vec<R>: FromIterator<R::Output>,
{
}
