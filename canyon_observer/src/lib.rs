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
pub mod migrations;

extern crate canyon_crud;

mod constants;
pub mod manager;

use crate::migrations::register_types::CanyonRegisterEntity;
use canyon_connection::lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};

pub static CANYON_REGISTER_ENTITIES: Mutex<Vec<CanyonRegisterEntity<'static>>> =
    Mutex::new(Vec::new());
lazy_static! {
    pub static ref QUERIES_TO_EXECUTE: Mutex<HashMap<&'static str, Vec<String>>> =
        Mutex::new(HashMap::new());
}
