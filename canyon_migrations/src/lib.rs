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

extern crate canyon_connection;
extern crate canyon_crud;
extern crate canyon_entities;

mod constants;

use canyon_connection::lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};

lazy_static! {
    pub static ref QUERIES_TO_EXECUTE: Mutex<HashMap<String, Vec<String>>> =
        Mutex::new(HashMap::new());
    pub static ref CM_QUERIES_TO_EXECUTE: Mutex<HashMap<String, Vec<String>>> =
        Mutex::new(HashMap::new());
}

/// Stores a newly generated SQL statement from the migrations into the register
pub fn save_migrations_query_to_execute(stmt: String, ds_name: &str) {
    if QUERIES_TO_EXECUTE.lock().unwrap().contains_key(ds_name) {
        QUERIES_TO_EXECUTE
            .lock()
            .unwrap()
            .get_mut(ds_name)
            .unwrap()
            .push(stmt);
    } else {
        QUERIES_TO_EXECUTE
            .lock()
            .unwrap()
            .insert(ds_name.to_owned(), vec![stmt]);
    }
}
