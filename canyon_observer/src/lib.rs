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


pub mod handler;
mod memory;
mod constants;

// Database Engine related
pub mod postgresql;

extern crate canyon_crud;

use std::sync::Mutex;
use lazy_static::lazy_static;

use crate::postgresql::register_types::CanyonRegisterEntity;


lazy_static! {  // TODO implement an access control polity by number of times read the static refs
    pub static ref CANYON_REGISTER_ENTITIES: Mutex<Vec<CanyonRegisterEntity<'static>>> = Mutex::new(Vec::new());
    pub static ref QUERIES_TO_EXECUTE: Mutex<Vec<String>> = Mutex::new(Vec::new());
}
