use crate::connection::contracts::DbConnection;
use crate::mapper::RowMapper;
use crate::rows::FromSqlOwnedValue;
use crate::{query::parameters::QueryParameter, rows::CanyonRows};
use std::error::Error;
use std::future::Future;
use crate::query::querybuilder::QueryBuilderOps;

/// The `Transaction` trait serves as a proxy for types implementing CRUD operations.
///
/// This trait provides a set of static methods that mirror the functionality of CRUD operations,
/// allowing implementors to be coerced into `<#ty as Transaction>::...` usage patterns.
/// It is primarily used by the generated macros of `CrudOperations` to simplify interaction
/// with database entities by abstracting common operations such as querying rows, executing
/// statements, and retrieving single results.
///
/// # Purpose
/// The `Transaction` trait is typically used to provide a unified interface for CRUD operations
/// on database entities. It enables developers to work with any type that implements the required
/// CRUD traits, abstracting away the underlying database connection details.
///
/// # Features
/// - Acts as a proxy for CRUD operations.
/// - Provides static methods for common database entity operations.
/// - Simplifies interaction with database entities.
///
/// # Examples
/// ```ignore
/// async fn perform_query<E: CrudOperations + Send>(entity: E) {
///     let result = <E as Transaction>::query("SELECT * FROM users", &[], entity).await;
///     match result {
///         Ok(rows) => println!("Retrieved {} rows", rows.len()),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
///
/// # Methods
/// - `query`: Executes a query and retrieves multiple rows mapped to a user-defined type.
/// - `query_one`: Executes a query and retrieves a single row mapped to a user-defined type.
/// - `query_one_for`: Executes a query and retrieves a single value of a specific type.
/// - `query_rows`: Executes a query and retrieves the raw rows wrapped in `CanyonRows`.
/// - `execute`: Executes a SQL statement and returns the number of affected rows.
pub trait Transaction {
    fn query<S, R>(
        stmt: S,
        params: &[&dyn QueryParameter],
        input: impl DbConnection + Send,
    ) -> impl Future<Output = Result<Vec<R>, Box<dyn Error + Send + Sync>>>
    where
        S: AsRef<str> + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        async move { input.query(stmt, params).await }
    }

    fn query_one<'a, S, Z, R>(
        stmt: S,
        params: Z,
        input: impl DbConnection + Send + 'a,
    ) -> impl Future<Output = Result<Option<R::Output>, Box<dyn Error + Send + Sync>>> + Send
    where
        S: AsRef<str> + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter]> + Send,
        R: RowMapper,
    {
        async move { input.query_one::<R>(stmt.as_ref(), params.as_ref()).await }
    }

    fn query_one_for<'a, S, Z, F: FromSqlOwnedValue<F>>(
        stmt: S,
        params: Z,
        input: impl DbConnection + Send + 'a,
    ) -> impl Future<Output = Result<F, Box<dyn Error + Send + Sync>>> + Send
    where
        S: AsRef<str> + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter]> + Send + 'a,
    {
        async move { input.query_one_for(stmt.as_ref(), params.as_ref()).await }
    }

    fn query_one_for_with_querybuilder<'a, T, F: FromSqlOwnedValue<F>>(
        builder: T,
        input: impl DbConnection + Send + 'a,
    ) -> impl Future<Output = Result<F, Box<dyn Error + Send + Sync>>> + Send
    where
        T: QueryBuilderOps<'a>,
    {
        let query = builder.build()?;
        async move { input.query_one_for(query.sql(), query.).await }
    }

    /// Performs a query against the targeted database by the selected or
    /// the defaulted datasource, wrapping the resultant collection of entities
    /// in [`super::rows::CanyonRows`]
    fn query_rows<'a, S, Z>(
        stmt: S,
        params: Z,
        input: impl DbConnection + Send + 'a,
    ) -> impl Future<Output = Result<CanyonRows, Box<dyn Error + Send + Sync>>> + Send
    where
        S: AsRef<str> + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter]> + Send + 'a,
    {
        async move { input.query_rows(stmt.as_ref(), params.as_ref()).await }
    }

    fn execute<'a, S, Z>(
        stmt: S,
        params: Z,
        input: impl DbConnection + Send + 'a,
    ) -> impl Future<Output = Result<u64, Box<dyn Error + Send + Sync>>> + Send
    where
        S: AsRef<str> + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter]> + Send + 'a,
    {
        async move { input.execute(stmt.as_ref(), params.as_ref()).await }
    }
}
