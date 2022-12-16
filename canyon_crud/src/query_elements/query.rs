use std::{fmt::Debug, marker::PhantomData};

use crate::{
    bounds::QueryParameters,
    crud::{CrudOperations, Transaction},
    mapper::RowMapper
};

/// Holds a sql sentence details
#[derive(Debug)]
pub struct Query<'a, T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>> {
    pub sql: String,
    pub params: Vec<&'a dyn QueryParameters<'a>>,
    marker: PhantomData<T>,
}

impl<'a, T> Query<'a, T>
where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>,
{
    pub fn new(sql: String, datasource_name: &'a str) -> Query<'a, T> {
        Self {sql, params: vec![], marker: PhantomData}
    }
}
