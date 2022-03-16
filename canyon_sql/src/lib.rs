// Common reexports (dependencies)
pub use tokio;
pub use async_trait;
pub use tokio_postgres;
// pub use canyon_observer;

// Core mods
mod connector;
mod results;

// Core public mods
pub mod crud;
pub mod mapper;
pub mod handler;

// Macros crate
pub extern crate canyon_macros;
pub use canyon_observer;


/// This reexports allows the users to import all the available
/// `Canyon-SQL` features in a single statement like:
/// 
/// `use canyon_sql::*`
/// 
/// and avoids polluting the macros with imports.
/// 
/// The decision of reexports all this crates was made because the macros
/// was importing this ones already, but if two structures was defined on the 
/// same file, the imported names into it collinding, avoiding let the user
/// to have multiple structs in only one file.
/// 
/// This particular feature (or decision) will be opened for revision
/// 'cause it's not definitive to let this forever
pub use canyon_macros::*;
pub use crud::*;
pub use mapper::*;
pub use async_trait::*;
pub use tokio_postgres::Row;