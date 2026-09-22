//! The core module of Canyon-SQL.
//!
//! This module provides the foundational components for database connections, query execution,
//! and data mapping. It includes support for multiple database backends such as PostgreSQL,
//! MySQL, and SQL Server, and defines traits and utilities for interacting with these databases.

#[cfg(not(any(feature = "postgres", feature = "mysql", feature = "mssql")))]
compile_error!(
    "Canyon-SQL requires at least one SQL backend feature: `postgres`, `mysql` or `mssql`."
);

#[cfg(feature = "postgres")]
pub extern crate tokio_postgres;

#[cfg(feature = "mssql")]
pub extern crate async_std;
#[cfg(feature = "mssql")]
pub extern crate tiberius;

#[cfg(feature = "mysql")]
pub extern crate mysql_async;

extern crate core;

#[cfg(any(feature = "postgres", feature = "mysql", feature = "mssql"))]
pub mod canyon;

#[cfg(any(feature = "postgres", feature = "mysql", feature = "mssql"))]
pub mod column;
#[cfg(any(feature = "postgres", feature = "mysql", feature = "mssql"))]
pub mod connection;
#[cfg(any(feature = "postgres", feature = "mysql", feature = "mssql"))]
pub mod error;
#[cfg(any(feature = "postgres", feature = "mysql", feature = "mssql"))]
pub mod mapper;
#[cfg(any(feature = "postgres", feature = "mysql", feature = "mssql"))]
pub mod query;
#[cfg(any(feature = "postgres", feature = "mysql", feature = "mssql"))]
pub mod row;
#[cfg(any(feature = "postgres", feature = "mysql", feature = "mssql"))]
pub mod rows;
#[cfg(any(feature = "postgres", feature = "mysql", feature = "mssql"))]
pub mod transaction;
