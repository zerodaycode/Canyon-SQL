use crate::query_elements::query_builder::{
    DeleteQueryBuilder, SelectQueryBuilder, UpdateQueryBuilder,
};
use canyon_core::connection::db_connector::DbConnection;
use canyon_core::query_parameters::QueryParameter;
use canyon_core::{mapper::RowMapper, transaction::Transaction};
use std::error::Error;
use std::future::Future;

/// *CrudOperations* it's the core part of Canyon-SQL.
///
/// Here it's defined and implemented every CRUD operation
/// that the user has available, just by deriving the `CanyonCrud`
/// derive macro when a struct contains the annotation.
///
/// Also, this traits needs that the type T over what it's generified
/// to implement certain types in order to work correctly.
///
/// The most notorious one it's the [`RowMapper<T>`] one, which allows
/// Canyon to directly maps database results into structs.
///
/// See it's definition and docs to see the implementations.
/// Also, you can find the written macro-code that performs the auto-mapping
/// in the *canyon_sql_root::canyon_macros* crates, on the root of this project.
pub trait CrudOperations<T>: Transaction<T>
where
    T: CrudOperations<T> + RowMapper<T>,
{
    fn find_all() -> impl Future<Output = Result<Vec<T>, Box<(dyn Error + Sync + Send)>>> + Send;

    fn find_all_with<'a, I>(
        input: I,
    ) -> impl Future<Output = Result<Vec<T>, Box<(dyn Error + Sync + Send + 'a)>>> + Send
    where
        I: DbConnection + Send + 'a;

    fn find_all_unchecked() -> impl Future<Output = Vec<T>> + Send;

    fn find_all_unchecked_with<'a, I>(input: I) -> impl Future<Output = Vec<T>> + Send
    where
        I: DbConnection + Send + 'a;

    fn select_query<'a>() -> SelectQueryBuilder<'a, T, &'a str>;

    fn select_query_with<'a, I>(input: I) -> SelectQueryBuilder<'a, T, I>
    where
        I: DbConnection + Send + 'a;

    fn count() -> impl Future<Output = Result<i64, Box<(dyn Error + Sync + Send)>>> + Send;

    fn count_with<'a, I>(
        input: I,
    ) -> impl Future<Output = Result<i64, Box<(dyn Error + Sync + Send + 'a)>>> + Send
    where
        I: DbConnection + Send + 'a;

    fn find_by_pk<'a>(
        value: &'a dyn QueryParameter<'a>,
    ) -> impl Future<Output = Result<Option<T>, Box<(dyn Error + Sync + Send + 'a)>>> + Send;

    fn find_by_pk_with<'a, I>(
        value: &'a dyn QueryParameter<'a>,
        input: I,
    ) -> impl Future<Output = Result<Option<T>, Box<(dyn Error + Sync + Send + 'a)>>> + Send
    where
        I: DbConnection + Send + 'a;

    // fn insert<'a>(
    //     &'a mut self,
    // ) -> impl Future<Output = Result<(), Box<(dyn Error + Sync + Send + 'a)>>> + Send;
    //
    // fn insert_with<'a, I>(
    //     &mut self,
    //     input: I,
    // ) -> impl Future<Output = Result<(), Box<(dyn Error + Sync + Send + 'a)>>> + Send
    // where
    //     I: DbConnection + Send + 'a;
    //
    // fn multi_insert<'a>(
    //     instances: &'a mut [&'a mut T],
    // ) -> impl Future<Output = Result<(), Box<(dyn Error + Sync + Send + 'a)>>> + Send;
    //
    // fn multi_insert_with<'a, I>(
    //     instances: &'a mut [&'a mut T],
    //     input: I,
    // ) -> impl Future<Output = Result<(), Box<(dyn Error + Sync + Send + 'a)>>> + Send
    // where
    //     I: DbConnection + Send + 'a;
    //
    // fn update(&self) -> impl Future<Output = Result<(), Box<(dyn Error + Sync + Send)>>> + Send;
    //
    // fn update_with<'a, I>(
    //     &self,
    //     input: I,
    // ) -> impl Future<Output = Result<(), Box<(dyn Error + Sync + Send + 'a)>>> + Send
    // where
    //     I: DbConnection + Send + 'a;
    //
    // fn update_query<'a>() -> UpdateQueryBuilder<'a, T, &'a str>;
    //
    // fn update_query_with<'a, I>(input: I) -> UpdateQueryBuilder<'a, T, I>
    // where
    //     I: DbConnection + Send + 'a;
    //
    // fn delete(&self) -> impl Future<Output = Result<(), Box<(dyn Error + Sync + Send)>>> + Send;
    //
    // fn delete_with<'a, I>(
    //     &self,
    //     input: I,
    // ) -> impl Future<Output = Result<(), Box<(dyn Error + Sync + Send + 'a)>>> + Send
    // where
    //     I: DbConnection + Send + 'a;
    //
    // fn delete_query<'a>() -> DeleteQueryBuilder<'a, T, &'a str>;
    //
    // fn delete_query_with<'a, I>(input: I) -> DeleteQueryBuilder<'a, T, I>
    // where
    //     I: DbConnection + Send + 'a;
}
