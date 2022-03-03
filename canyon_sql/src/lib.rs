// Common reexports (dependencies)
pub use tokio;
pub use async_trait;
pub use tokio_postgres;

// Core mods
mod connector;
mod results;

// Core public mods
pub mod crud;
pub mod mapper;

// Macros crate
pub extern crate canyon_macros;

/// This reexports allows the users to import all the available
/// `Canyon-SQL` features in a single statement like:
/// 
/// `use canyon_sql::*`
/// 
/// 
pub use canyon_macros::*;
pub use crud::*;
pub use mapper::*;
pub use async_trait::*;
pub use tokio_postgres::Row;