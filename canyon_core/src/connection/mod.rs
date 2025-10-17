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

pub mod clients;
pub mod conn_errors;
pub mod contracts;
pub mod database_type;
pub mod datasources;
pub mod db_connector;
pub mod pool;
use crate::canyon::Canyon;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

//
// // TODO's: DatabaseConnection and DataSource can implement default, so there's no need to use str and &str
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
