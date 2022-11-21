/// Holds the data needed by Canyon when the user
/// application it's running.
///
/// Takes care about provide a namespace where retrieve the
/// database credentials in only one place
///
/// Takes care about track what data structures Canyon
/// should be managing
///
/// Takes care about the queries that Canyon has to execute
/// in order to perform the migrations
pub mod postgresql;

extern crate canyon_crud;

// The migrator tool
mod constants;
mod memory;
pub mod handler;
pub mod manager;

use std::sync::Mutex;
use crate::postgresql::register_types::CanyonRegisterEntity;

pub static CANYON_REGISTER_ENTITIES: Mutex<Vec<CanyonRegisterEntity<'static>>> =
    Mutex::new(Vec::new());
pub static QUERIES_TO_EXECUTE: Mutex<Vec<String>> = Mutex::new(Vec::new());
