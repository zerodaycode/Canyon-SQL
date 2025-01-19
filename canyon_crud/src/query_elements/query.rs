use std::{fmt::Debug, marker::PhantomData};

use crate::crud::CrudOperations;
use canyon_core::{mapper::RowMapper, query::Transaction, query_parameters::QueryParameter};

/// Holds a sql sentence details
#[derive(Debug, Clone)]
pub struct Query<'a, T: CrudOperations<T> + Transaction<T> + RowMapper<T>> {
    pub sql: String,
    pub params: Vec<&'a dyn QueryParameter<'a>>,
    marker: PhantomData<T>,
}

impl<'a, T> Query<'a, T>
where
    T: CrudOperations<T> + Transaction<T> + RowMapper<T>,
{
    pub fn new(sql: String) -> Query<'a, T> {
        Self {
            sql,
            params: vec![],
            marker: PhantomData,
        }
    }
}
