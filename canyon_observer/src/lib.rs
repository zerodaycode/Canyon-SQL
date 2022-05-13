pub mod handler;
mod memory;
mod constants;

// Database Engine related
pub mod postgresql;

extern crate canyon_crud;

// use lazy_static::lazy_static;

use crate::postgresql::register_types::CanyonRegisterEntity;

/// Holds the data needed by Canyon when the user
/// application it's running.
/// 
/// Takes care about provide a namespace where retrieve the
/// database credentials in only one place
/// 
/// Also takes care about track what data structures Canyon
/// should be managing
pub static mut QUERIES_TO_EXECUTE: Vec<String> = Vec::new();
pub static mut CANYON_REGISTER_ENTITIES: Vec<CanyonRegisterEntity> = Vec::new();

// lazy_static! {
    // static ref CREDENTIALS: DatabaseCredentials = DatabaseCredentials::new();
// }

// TODO Change it for lazy_static! elements
