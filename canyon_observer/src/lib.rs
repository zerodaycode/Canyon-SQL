pub mod credentials;
pub mod handler;
mod memory;
mod constants;

// Database Engine related
pub mod postgresql;

extern crate canyon_crud;

use postgresql::register_types::CanyonRegisterEntity;
// use credentials::DatabaseCredentials;

/// Holds the data needed by Canyon when the host
/// application it's running.
/// 
/// Takes care about provide a namespace where retrieve the
/// database credentials in only one place
/// 
/// Also takes care about track what data structures Canyon
/// should be managing
pub static mut QUERIES_TO_EXECUTE: Vec<String> = Vec::new();
pub static mut CANYON_REGISTER_ENTITIES: Vec<CanyonRegisterEntity> = Vec::new();

// TODO Change it for lazy_static! elements
