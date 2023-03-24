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
    pub static ref CM_QUERIES_TO_EXECUTE: Mutex<HashMap<&'static str, Vec<(&'static str, Vec<String>)>>> = // TODO Provisional until we parameterize as well the migration queries
        Mutex::new(HashMap::new());
}

// TODO replace the unwraps for the operator ? when the appropiated crate will be added
pub fn add_cm_query_to_execute(stmt: &'static str, datasource_name: &'static str, params: Vec<String>) {
    if CM_QUERIES_TO_EXECUTE
        .lock()
        .unwrap()
        .contains_key(datasource_name)
    {
        CM_QUERIES_TO_EXECUTE
            .lock()
            .unwrap()
            .get_mut(datasource_name)
            .unwrap()
            .push((stmt, params));
    } else {
        CM_QUERIES_TO_EXECUTE
            .lock()
            .unwrap()
            .insert(datasource_name, vec![(stmt, params)]);
    }
}