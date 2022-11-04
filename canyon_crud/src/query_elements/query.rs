use std::{fmt::Debug, marker::PhantomData};

use crate::{
    bounds::QueryParameters,
    crud::{CrudOperations, Transaction},
    mapper::RowMapper,
    query_elements::query_builder::QueryBuilder,
};

/// Holds a sql sentence details
#[derive(Clone)]
pub struct Query<'a, T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>> {
    pub sql: String,
    pub params: &'a [&'a dyn QueryParameters<'a>],
    marker: PhantomData<T>,
}

impl<'a, T> Query<'a, T>
where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>,
{
    pub fn generate(sql: String, datasource_name: &'a str) -> QueryBuilder<'a, T> {
        let self_ = Self {
            sql,
            params: &[],
            marker: PhantomData,
        };
        QueryBuilder::<T>::new(self_, datasource_name)
    }
}
