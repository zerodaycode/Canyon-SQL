#![cfg(feature = "migrations")]
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
extern crate canyon_entities;

mod constants;

use std::sync::OnceLock;
use std::{collections::HashMap, sync::Mutex};

pub static QUERIES_TO_EXECUTE: OnceLock<Mutex<HashMap<String, Vec<String>>>> = OnceLock::new();
pub static CM_QUERIES_TO_EXECUTE: OnceLock<Mutex<HashMap<String, Vec<String>>>> = OnceLock::new();

/// Stores a newly generated SQL statement from the migrations into the register
pub fn save_migrations_query_to_execute(stmt: String, ds_name: &str) {
    // Access the QUERIES_TO_EXECUTE hash map and lock it for safe access
    let queries_to_execute = QUERIES_TO_EXECUTE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut queries = queries_to_execute.lock().unwrap();

    if queries.contains_key(ds_name) {
        queries.get_mut(ds_name).unwrap().push(stmt);
    } else {
        queries.insert(ds_name.to_owned(), vec![stmt]);
    }
}
