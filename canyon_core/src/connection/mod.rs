//! The connection module of Canyon-SQL.
//!
//! This module handles database connections, including connection pooling and configuration.
//! It provides abstractions for managing multiple datasources and supports asynchronous operations.

#[cfg(feature = "postgres")]
pub extern crate tokio_postgres;

#[cfg(feature = "mssql")]
pub extern crate async_std;
#[cfg(feature = "mssql")]
pub extern crate tiberius;

#[cfg(feature = "mysql")]
pub extern crate mysql_async;

pub extern crate futures;
pub extern crate tokio;
pub extern crate tokio_util;

#[macro_use]
pub mod impl_db_connection_macro;

pub mod clients;
pub mod conn_errors;
pub mod contracts;
pub mod database_type;
pub mod datasources;
pub mod db_connector;

use crate::canyon::Canyon;
use crate::connection::contracts::DbConnection;
use crate::connection::database_type::DatabaseType;

use std::error::Error;
use std::sync::{Arc, OnceLock};

use tokio::runtime::Runtime;
use tokio::sync::Mutex;

// // TODO's: DatabaseConnector and DataSource can implement default, so there's no need to use str and &str
// // as defaults anymore, since the can load as the default the first one defined in the config file, or have more
// // complex workflows that are deferred to initialization time
//
// // TODO: Crud Operations should be split into two different derives, splitting the automagic from the _with ones

pub(crate) static CANYON_INSTANCE: OnceLock<Canyon> = OnceLock::new();

// Use OnceLock for the Tokio runtime
static CANYON_TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

// Function to get the runtime (lazy initialization)
pub fn get_canyon_tokio_runtime() -> &'static Runtime {
    CANYON_TOKIO_RUNTIME
        .get_or_init(|| Runtime::new().expect("Failed initializing the Canyon-SQL Tokio Runtime"))
}

use crate::mapper::RowMapper;
use crate::query::parameters::QueryParameter;
use crate::rows::{CanyonRows, FromSqlOwnedValue};

// Apply the macro to implement DbConnection for &str and str
use crate::impl_db_connection_for_str;
impl_db_connection_for_str!(str);
impl_db_connection_for_str!(&str);

impl<T> DbConnection for Arc<Mutex<T>>
where
    T: DbConnection + Send,
    Self: Clone,
{
    async fn query_rows(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
        self.lock().await.query_rows(stmt, params).await
    }

    async fn query<S, R>(
        &self,
        stmt: S,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<Vec<R>, Box<dyn Error + Send + Sync>>
    where
        S: AsRef<str> + Send,
        R: RowMapper,
        Vec<R>: FromIterator<R::Output>,
    {
        self.lock().await.query(stmt, params).await
    }

    async fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
    where
        R: RowMapper,
    {
        self.lock().await.query_one::<R>(stmt, params).await
    }

    async fn query_one_for<F: FromSqlOwnedValue<F>>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<F, Box<dyn Error + Send + Sync>> {
        self.lock().await.query_one_for::<F>(stmt, params).await
    }

    async fn execute(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<u64, Box<dyn Error + Send + Sync>> {
        self.lock().await.execute(stmt, params).await
    }

    async fn get_database_type(&self) -> Result<DatabaseType, Box<dyn Error + Send + Sync>> {
        todo!()
    }
}
