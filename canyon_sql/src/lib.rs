///! The root crate of the `Canyon-SQL` project.
/// 
/// Here it's where all the available functionalities and features
/// reaches the top most level, grouping them and making them visible
/// through this crate, building the *public API* of the library


/// Reexported elements to the root of the public API
// pub use canyon_crud::*;
pub use canyon_observer::*;

/// Public API for the `Canyon-SQL` proc-macros, and for the external ones
pub mod macros {
    pub use canyon_macros::*;
    pub use async_trait::*;
}

/// Crud module serves to reexport the public elements of the `canyon_crud` crate,
/// exposing them through the public API
pub mod crud {
    pub use canyon_crud::crud::*;
    pub use canyon_crud::mapper::*;
    pub use canyon_crud::bounds;
    pub use canyon_crud::result::*;
    pub use canyon_crud::DatabaseType;
}

/// Re-exports the query elements from the `crud`crate
pub mod query {
    pub use canyon_crud::query_elements::{query::*, query_builder::*};
    pub use canyon_crud::query_elements::operators;
}

/// Reexport the available database clients within Canyon
pub mod db_clients {
    pub use canyon_connection::tokio_postgres;
    pub use canyon_connection::tiberius;
}

/// Reexport the needed runtime dependencies
pub mod runtime {
    pub use canyon_connection::CANYON_TOKIO_RUNTIME;
    pub use canyon_connection::init_connection_cache;
    pub use canyon_connection::tokio;
    pub use canyon_connection::tokio_util;
    pub use canyon_connection::futures;
}

/// Module for reexport the `chrono` crate with the allowed public and available types in Canyon
pub mod date_time {
    pub use canyon_crud::chrono::{
        DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc,
    };
}
