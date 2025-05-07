use std::fmt::Debug;

use crate::query::parameters::QueryParameter;

// TODO: all the query works here
// TODO: exports things like Select::... where receives the table
// name and prepares the raw query (maybe with const_format!) for improved performance

// TODO: query should implement ToStatement (as the drivers underneath Canyon) or similar
// to be usable directly in the input of Transaction and DbConnenction
/// Holds a sql sentence details
#[derive(Debug, Clone)]
pub struct Query<'a> {
    pub sql: String,
    pub params: Vec<&'a dyn QueryParameter<'a>>,
}

impl<'a> Query<'a> {
    pub fn new(sql: String) -> Query<'a> {
        Self {
            sql,
            params: vec![],
        }
    }
}
