use canyon_core::{
    connection::{contracts::DbConnection, database_type::DatabaseType},
    error::CanyonResult,
    mapper::RowMapper,
    query::{
        parameters::QueryParameter,
        querybuilder::{DeleteQueryBuilder, SelectQueryBuilder, UpdateQueryBuilder},
    },
};
use std::future::Future;

pub trait ReadOperations<R>: Send
where
    R: RowMapper,
    Vec<R>: FromIterator<R::Output>,
{
    fn find_all() -> impl Future<Output = CanyonResult<Vec<R>>> + Send;

    fn find_all_with<'connection, I>(input: I) -> impl Future<Output = CanyonResult<Vec<R>>> + Send
    where
        I: DbConnection + Send + 'connection;

    fn select_query<'a>() -> CanyonResult<SelectQueryBuilder<'a>>;

    fn select_query_with<'a>(database_type: DatabaseType) -> CanyonResult<SelectQueryBuilder<'a>>;

    fn count() -> impl Future<Output = CanyonResult<i64>> + Send;

    fn count_with<'connection, I>(input: I) -> impl Future<Output = CanyonResult<i64>> + Send
    where
        I: DbConnection + Send + 'connection;

    fn find_by_pk(
        value: &dyn QueryParameter,
    ) -> impl Future<Output = CanyonResult<Option<R>>> + Send;

    fn find_by_pk_with<'value, I>(
        value: &'value dyn QueryParameter,
        input: I,
    ) -> impl Future<Output = CanyonResult<Option<R>>> + Send
    where
        I: DbConnection + Send + 'value;
}

pub trait InsertOperations: Send {
    fn insert(&mut self) -> impl Future<Output = CanyonResult<()>> + Send;

    fn insert_with<'connection, I>(
        &mut self,
        input: I,
    ) -> impl Future<Output = CanyonResult<()>> + Send
    where
        I: DbConnection + Send + 'connection;
}

pub trait UpdateOperations: Send {
    fn update(&self) -> impl Future<Output = CanyonResult<u64>> + Send;

    fn update_with<'connection, I>(
        &self,
        input: I,
    ) -> impl Future<Output = CanyonResult<u64>> + Send
    where
        I: DbConnection + Send + 'connection;

    fn update_query<'canyon>() -> CanyonResult<UpdateQueryBuilder<'canyon>>;

    fn update_query_with<'a>(database_type: DatabaseType) -> UpdateQueryBuilder<'a>;
}

pub trait DeleteOperations: Send {
    fn delete(&self) -> impl Future<Output = CanyonResult<()>> + Send;

    fn delete_with<'connection, I>(
        &self,
        input: I,
    ) -> impl Future<Output = CanyonResult<()>> + Send
    where
        I: DbConnection + Send + 'connection;

    fn delete_query<'canyon>() -> CanyonResult<DeleteQueryBuilder<'canyon>>;

    fn delete_query_with<'a>(database_type: DatabaseType) -> DeleteQueryBuilder<'a>;
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
