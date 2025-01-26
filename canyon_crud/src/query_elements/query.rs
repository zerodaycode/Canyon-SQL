use std::fmt::Debug;

use canyon_core::query_parameters::QueryParameter;

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
