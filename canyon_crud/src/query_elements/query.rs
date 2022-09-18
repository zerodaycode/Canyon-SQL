use canyon_connection::{tokio_postgres::types::ToSql, tiberius::IntoSql};
use std::{fmt::Debug, marker::PhantomData};

use crate::{
    query_elements::query_builder::QueryBuilder,
    crud::{Transaction, CrudOperations}, mapper::RowMapper, bounds::QueryParameters 
};



/// Holds a sql sentence details
#[derive(Clone)]
pub struct Query<'a, T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>> {
    pub sql: String,
    pub params: &'a[&'a dyn QueryParameters<'a>],
    marker: PhantomData<T>
}

impl<'a, T> Query<'a, T> 
    where
        T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> 
{
    pub fn new(sql: String, params: &'a[&'a dyn QueryParameters<'a>], datasource_name: &'a str) -> QueryBuilder<'a, T> {
        let self_ = Self {
            sql: sql,
            params: params,
            marker: PhantomData
        };
        QueryBuilder::<T>::new(self_, datasource_name)
    }
}

