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

/// Read operations generated for an entity or mapped repository.
pub trait Read<R>: Send
where
    R: RowMapper,
    Vec<R>: FromIterator<R::Output>,
{
    /// Retrieves every row using the default datasource.
    fn find_all() -> impl Future<Output = CanyonResult<Vec<R>>> + Send;

    /// Retrieves every row using the supplied connection or datasource.
    fn find_all_with<'connection, I>(input: I) -> impl Future<Output = CanyonResult<Vec<R>>> + Send
    where
        I: DbConnection + Send + 'connection;

    /// Creates a select query builder targeting the default datasource.
    fn select_query<'a>() -> CanyonResult<SelectQueryBuilder<'a>>;

    /// Creates a select query builder for the specified database dialect.
    fn select_query_with<'a>(database_type: DatabaseType) -> CanyonResult<SelectQueryBuilder<'a>>;

    /// Counts all rows using the default datasource.
    fn count() -> impl Future<Output = CanyonResult<i64>> + Send;

    /// Counts all rows using the supplied connection or datasource.
    fn count_with<'connection, I>(input: I) -> impl Future<Output = CanyonResult<i64>> + Send
    where
        I: DbConnection + Send + 'connection;

    /// Retrieves the row matching `value` in the entity's primary-key column.
    fn find_by_pk(
        value: &dyn QueryParameter,
    ) -> impl Future<Output = CanyonResult<Option<R>>> + Send;

    /// Retrieves a row by primary key using the supplied connection or datasource.
    fn find_by_pk_with<'value, I>(
        value: &'value dyn QueryParameter,
        input: I,
    ) -> impl Future<Output = CanyonResult<Option<R>>> + Send
    where
        I: DbConnection + Send + 'value;
}

/// Insert operations generated for an entity instance.
pub trait Insert: Send {
    /// Inserts this entity using the default datasource.
    fn insert(&mut self) -> impl Future<Output = CanyonResult<()>> + Send;

    /// Inserts this entity using the supplied connection or datasource.
    fn insert_with<'connection, I>(
        &mut self,
        input: I,
    ) -> impl Future<Output = CanyonResult<()>> + Send
    where
        I: DbConnection + Send + 'connection;
}

/// Update operations generated for an entity instance.
pub trait Update: Send {
    /// Updates this entity and returns the number of affected rows.
    fn update(&self) -> impl Future<Output = CanyonResult<u64>> + Send;

    /// Updates this entity through the supplied connection and returns the affected-row count.
    fn update_with<'connection, I>(
        &self,
        input: I,
    ) -> impl Future<Output = CanyonResult<u64>> + Send
    where
        I: DbConnection + Send + 'connection;

    /// Creates an update query builder targeting the default datasource.
    fn update_query<'canyon>() -> CanyonResult<UpdateQueryBuilder<'canyon>>;

    /// Creates an update query builder for the specified database dialect.
    fn update_query_with<'a>(database_type: DatabaseType) -> UpdateQueryBuilder<'a>;
}

/// Delete operations generated for an entity instance.
pub trait Delete: Send {
    /// Deletes this entity using the default datasource.
    fn delete(&self) -> impl Future<Output = CanyonResult<()>> + Send;

    /// Deletes this entity using the supplied connection or datasource.
    fn delete_with<'connection, I>(
        &self,
        input: I,
    ) -> impl Future<Output = CanyonResult<()>> + Send
    where
        I: DbConnection + Send + 'connection;

    /// Creates a delete query builder targeting the default datasource.
    fn delete_query<'canyon>() -> CanyonResult<DeleteQueryBuilder<'canyon>>;

    /// Creates a delete query builder for the specified database dialect.
    fn delete_query_with<'a>(database_type: DatabaseType) -> DeleteQueryBuilder<'a>;
}

/// Complete static CRUD contract.
///
/// Canyon implements this marker trait automatically when a type implements
/// [`Read`], [`Insert`], [`Update`] and [`Delete`].
pub trait Crud<R>: Read<R> + Insert + Update + Delete
where
    R: RowMapper,
    Vec<R>: FromIterator<R::Output>,
{
}

impl<T, R> Crud<R> for T
where
    T: Read<R> + Insert + Update + Delete,
    R: RowMapper,
    Vec<R>: FromIterator<R::Output>,
{
}
