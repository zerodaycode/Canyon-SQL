use crate::connection::database_type::DatabaseType;
use crate::mapper::RowMapper;
use crate::query_parameters::QueryParameter;
use crate::rows::{CanyonRows, FromSqlOwnedValue};
use std::error::Error;
use std::fmt::Display;
use std::future::Future;

mod r#impl; // contains the implementation details for the trait

/// The `DbConnection` trait defines the core functionality required for interacting with a database connection.
/// It provides methods for executing queries, retrieving rows, and obtaining metadata about the database type.
///
/// This trait is designed to be implemented by various database connection types, enabling a unified interface
/// for database operations. Each method is asynchronous and returns a `Future` to support non-blocking operations.
///
/// # Examples
///
/// ```ignore
/// use crate::connection::DbConnection;
///
/// async fn execute_query<C: DbConnection>(conn: &C) {
///     let result = conn.execute("INSERT INTO users (name) VALUES ($1)", &[&"John"]).await;
///     match result {
///         Ok(rows_affected) => println!("Rows affected: {}", rows_affected),
///         Err(e) => eprintln!("Error executing query: {}", e),
///     }
/// }
/// ```
///
/// # Required Methods
/// Each method in this trait must be implemented by the implementor.
pub trait DbConnection {
    /// Executes a query and retrieves multiple rows from the database.
    ///
    /// # Arguments
    /// * `stmt` - A SQL statement to execute.
    /// * `params` - A slice of query parameters to bind to the statement.
    ///
    /// # Returns
    /// A [Future] that resolves to a [Result] containing [`CanyonRows`] on success or an error on failure.
    fn query_rows<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn Error + Send + Sync)>>> + Send;

    /// Executes a query and maps the result to a collection of rows of type `R`.
    ///
    /// # Arguments
    /// * `stmt` - A SQL statement to execute.
    /// * `params` - A slice of query parameters to bind to the statement.
    ///
    /// # Returns
    /// A [Future] that resolves to a [Result] containing a `Vec<R>` on success or an error on failure.
    ///
    /// The `R` type must implement the [`RowMapper`] trait.
    fn query<'a, S, R>(
        &self,
        stmt: S,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> impl Future<Output = Result<Vec<R>, Box<(dyn Error + Send + Sync)>>> + Send
    where
        S: AsRef<str> + Display + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>;

    /// Executes a query and retrieves a single row mapped to type `R`.
    ///
    /// # Arguments
    /// * `stmt` - A SQL statement to execute.
    /// * `params` - A slice of query parameters to bind to the statement.
    ///
    /// # Returns
    /// A [Future] that resolves to a [Result] containing an `Option<R::Output>` on success or an error on failure.
    ///
    /// The `R` type must implement the [`RowMapper`] trait.
    fn query_one<'a, R>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<Option<R::Output>, Box<(dyn Error + Send + Sync)>>> + Send
    where
        R: RowMapper;

    /// Executes a query and retrieves a single value of type `T`.
    ///
    /// # Arguments
    /// * `stmt` - A SQL statement to execute.
    /// * `params` - A slice of query parameters to bind to the statement.
    ///
    /// # Returns
    /// A [Future] that resolves to a [Result] containing the value of type `T` on success or an error on failure.
    ///
    /// The `T` type must implement the [`FromSqlOwnedValue`] trait.
    fn query_one_for<'a, T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<T, Box<(dyn Error + Send + Sync)>>> + Send;

    /// Executes a SQL statement and returns the number of affected rows.
    ///
    /// # Arguments
    /// * `stmt` - A SQL statement to execute.
    /// * `params` - A slice of query parameters to bind to the statement.
    ///
    /// # Returns
    /// A [Future] that resolves to a [Result] containing the number of affected rows on success or an error on failure.
    fn execute<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<u64, Box<(dyn Error + Send + Sync)>>> + Send;

    /// Retrieves the type of the database associated with the connection.
    ///
    /// # Returns
    /// A `Result` containing the [`DatabaseType`] on success or an error on failure.
    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Send + Sync)>>;
}
