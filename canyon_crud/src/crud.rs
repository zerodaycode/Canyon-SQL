use async_trait::async_trait;
use canyon_core::query_parameters::QueryParameter;
use canyon_core::{mapper::RowMapper, query::Transaction};
use canyon_core::query::TransactionInput;
use crate::query_elements::query_builder::{
    DeleteQueryBuilder, SelectQueryBuilder, UpdateQueryBuilder,
};

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
#[async_trait]
pub trait CrudOperations<T>: Transaction<T>
where
    T: CrudOperations<T> + RowMapper<T>,
{
    async fn find_all() -> Result<Vec<T>, Box<(dyn std::error::Error + Send + Sync)>>;

    async fn find_all_with<'a, I>(input: I) -> Result<Vec<T>, Box<(dyn std::error::Error + Send + Sync + 'a)>>
    where I: Into<TransactionInput<'a>> + Sync + Send + 'a;

    async fn find_all_unchecked() -> Vec<T>;

    async fn find_all_unchecked_with<'a, I>(input: I) -> Vec<T>
        where I: Into<TransactionInput<'a>> + Sync + Send + 'a;

    fn select_query<'a, I>() -> SelectQueryBuilder<'a, T, I> where I: Into<TransactionInput<'a>> + Sync + Send + 'a, TransactionInput<'a>: From<&'a I>,;

    fn select_query_with<'a, I>(input: I) -> SelectQueryBuilder<'a, T, I>
        where I: Into<TransactionInput<'a>> + Sync + Send + 'a,
              TransactionInput<'a>: From<&'a I>,;

    async fn count() -> Result<i64, Box<(dyn std::error::Error + Send + Sync)>>;

    async fn count_with<'a, I>(
        input: I,
    ) -> Result<i64, Box<(dyn std::error::Error + Send + Sync + 'a)>>
    where I: Into<TransactionInput<'a>> + Sync + Send + 'a;

    async fn find_by_pk<'a>(
        value: &'a dyn QueryParameter<'a>,
    ) -> Result<Option<T>, Box<(dyn std::error::Error + Send + Sync + 'a)>>;

    async fn find_by_pk_with<'a, I>(
        value: &'a dyn QueryParameter<'a>,
        input: I,
    ) -> Result<Option<T>, Box<(dyn std::error::Error + Send + Sync + 'a)>>;

    // async fn insert<'a>(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send + 'a>>;

    // async fn insert_with<'a>(
    //     &mut self,
    //     datasource_name: &'a str,
    // ) -> Result<(), Box<dyn std::error::Error + Sync + Send + 'a>>;

    // async fn multi_insert<'a>(
    //     instances: &'a mut [&'a mut T],
    // ) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'a)>>;

    // async fn multi_insert_with<'a>(
    //     instances: &'a mut [&'a mut T],
    //     datasource_name: &'a str,
    // ) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'a)>>;

    // async fn update(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>>;

    // async fn update_with<'a>(
    //     &self,
    //     datasource_name: &'a str,
    // ) -> Result<(), Box<dyn std::error::Error + Sync + Send>>;

    // fn update_query<'a>() -> UpdateQueryBuilder<'a, T>;

    // fn update_query_with(datasource_name: &str) -> UpdateQueryBuilder<'_, T>;

    // async fn delete(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>>;

    // async fn delete_with<'a>(
    //     &self,
    //     datasource_name: &'a str,
    // ) -> Result<(), Box<dyn std::error::Error + Sync + Send>>;

    // fn delete_query<'a>() -> DeleteQueryBuilder<'a, T>;

    // fn delete_query_with(datasource_name: &str) -> DeleteQueryBuilder<'_, T>;
}
