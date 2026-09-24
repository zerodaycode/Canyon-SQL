//! Static and runtime CRUD contracts used by Canyon-SQL's derive macros.

#![deny(missing_docs)]

/// CRUD operations performed directly by an entity.
pub mod crud;
/// CRUD operations performed by an adapter for a mapped entity.
pub mod entity;

pub use canyon_core::query::operators::*;

pub use canyon_core::connection::{database_type::DatabaseType, datasources::*};
pub use chrono;
