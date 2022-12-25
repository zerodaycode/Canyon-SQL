use std::{fmt::Debug, marker::PhantomData};

use crate::{
    bounds::QueryParameter,
    crud::{CrudOperations, Transaction},
    mapper::RowMapper,
};

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
