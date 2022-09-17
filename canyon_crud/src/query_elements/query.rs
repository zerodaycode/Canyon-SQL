use canyon_connection::{tokio_postgres::types::ToSql, tiberius::IntoSql};
use std::{fmt::Debug, marker::PhantomData};

use crate::{
    query_elements::query_builder::QueryBuilder,
    crud::{Transaction, CrudOperations}, mapper::RowMapper, bounds::QueryParameters 
};



/// Holds a sql sentence details
#[derive(Debug, Clone)]
pub struct Query<'a, W, T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>> 
    where W : ToSql + IntoSql<'a> + Clone + Sync + Send
{
    pub sql: String,
    pub params: &'a[W],
    marker: PhantomData<T>
}

impl<'a, W, T> Query<'a, W, T> 
    where 
        W: QueryParameters<'a>,
        T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> 
{
    pub fn new(sql: String, params: &'a[W], datasource_name: &'a str) -> QueryBuilder<'a, W, T> {
        let self_ = Self {
            sql: sql,
            params: params,
            marker: PhantomData
        };
        QueryBuilder::<W, T>::new(self_, datasource_name)
    }
}

