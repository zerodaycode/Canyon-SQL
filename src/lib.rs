//! The root crate of the `Canyon-SQL` project.
///
/// Here it's where all the available functionalities and features
/// reaches the top most level, grouping them and making them visible
/// through this crate, building the *public API* of the library
extern crate canyon_core;
extern crate canyon_crud;
extern crate canyon_macros;
#[cfg(feature = "migrations")]
extern crate canyon_migrations;

/// Reexported elements to the root of the public API
#[cfg(feature = "migrations")]
pub mod migrations {
    pub use canyon_migrations::migrations::{handler, processor};
}

/// The top level reexport. Here we define the path to some really important
/// things in `Canyon-SQL`, like the `main` macro, the IT macro.
pub use canyon_macros::main;

/// Public API for the `Canyon-SQL` proc-macros, and for the external ones
pub mod macros {
    pub use canyon_macros::*;
}

/// connection module serves to reexport the public elements of the `canyon_connection` crate,
/// exposing them through the public API
pub mod connection {
    pub use canyon_core::connection::database_type::DatabaseType;
    pub use canyon_core::connection::db_connector::DatabaseConnection;
    pub use canyon_core::connection::Canyon;
}

pub mod core {
    pub use canyon_core::connection::db_connector::DbConnection;
    pub use canyon_core::connection::Canyon;
    pub use canyon_core::mapper::*;
    pub use canyon_core::query_parameters::QueryParameter;
    pub use canyon_core::rows::CanyonRows;
    pub use canyon_core::transaction::Transaction;
}

/// Crud module serves to reexport the public elements of the `canyon_crud` crate,
/// exposing them through the public API
pub mod crud {
    pub use canyon_crud::bounds;
    pub use canyon_crud::crud::*;
}

/// Re-exports the query elements from the `crud`crate
pub mod query {
    pub use canyon_crud::query_elements::operators;
    pub use canyon_crud::query_elements::{query::*, query_builder::*};
}

/// Reexport the available database clients within Canyon
pub mod db_clients {
    #[cfg(feature = "mysql")]
    pub use canyon_core::connection::mysql_async;
    #[cfg(feature = "mssql")]
    pub use canyon_core::connection::tiberius;
    #[cfg(feature = "postgres")]
    pub use canyon_core::connection::tokio_postgres;
}

/// Reexport the needed runtime dependencies
pub mod runtime {
    pub use canyon_core::connection::futures;
    pub use canyon_core::connection::get_canyon_tokio_runtime;
    pub use canyon_core::connection::tokio;
    pub use canyon_core::connection::tokio_util;
}

/// Module for reexport the `chrono` crate with the allowed public and available types in Canyon
pub mod date_time {
    pub use canyon_crud::chrono::{
        DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc,
    };
}
