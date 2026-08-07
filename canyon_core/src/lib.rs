//! The core module of Canyon-SQL.
//!
//! This module provides the foundational components for database connections, query execution,
//! and data mapping. It includes support for multiple database backends such as PostgreSQL,
//! MySQL, and SQL Server, and defines traits and utilities for interacting with these databases.

#[cfg(feature = "postgres")]
pub extern crate tokio_postgres;

#[cfg(feature = "mssql")]
pub extern crate async_std;
#[cfg(feature = "mssql")]
pub extern crate tiberius;

#[cfg(feature = "mysql")]
pub extern crate mysql_async;

extern crate core;

pub mod canyon;

pub mod column;
pub mod connection;
pub mod mapper;
pub mod query;
pub mod row;
pub mod rows;
pub mod transaction;
