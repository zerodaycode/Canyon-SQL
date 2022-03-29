use tokio_postgres::types::ToSql;
use std::{fmt::Debug, marker::PhantomData};

use crate::{crud::{Transaction, CrudOperations}, result::DatabaseResult};

/// Holds a mut sql sentence
#[derive(Debug)]
pub struct Query<'a, T: Debug + Transaction<T>> {
    sql: String,
    params: &'a[Box<dyn ToSql + Sync>],
    marker: PhantomData<T>
}

// impl<'a, T: Debug + Transaction<T>> Transaction<Self> for T {}

impl<'a,T: Debug + CrudOperations<T> + Transaction<T>> Query<'a, T> {
    pub fn new(sql: String, params: &'a[Box<dyn ToSql + Sync>]) -> QueryBuilder<'a, T> {
        let self_ = Self {
            sql: sql,
            params: params,
            marker: PhantomData
        };
        QueryBuilder::<T>::new(self_)
    }
}

/// Builder for a query while chaining SQL clauses
#[derive(Debug)]
pub struct QueryBuilder<'a, T: Debug + Transaction<T>> {
    query: Query<'a, T>,
    where_clause: &'a str,
    // and_clause: &'a str,
    // in_clause: &'a[Box<dyn ToSql>],
    // order_by_clause: &'a str
}
impl<'a, T: Debug + Transaction<T>> QueryBuilder<'a, T> {

    // Generates a Query object that contains the necessary data to performn a query
    pub async fn query(mut self) -> DatabaseResult<T> {
        if self.where_clause != "" {
            self.query.sql.push_str(self.where_clause)
        }
        
        let mut unboxed_params = Vec::new();
        for element in self.query.params {
            unboxed_params.push(&**element);
        }

        T::query(&self.query.sql[..], &unboxed_params).await
    }

    pub fn new(query: Query<'a, T>) -> Self {
        Self {
            query: query,
            where_clause: "",
            // and_clause: "",
            // in_clause: &[],
            // order_by_clause: ""
        }
    }

    pub fn where_clause(mut self, r#where: &'a str) -> Self {
        self.where_clause = r#where;
        self
    } 

    // pub fn r#and(mut self, r#and: &'a str) -> &'a mut Self {
    //     self.and_clause = and;
    //     self
    // } 

    // pub fn r#in(mut self, in_values: &'a[Box<dyn ToSql>]) -> Self {
    //     self.in_clause = in_values;
    //     self
    // } 

    // pub fn order_by(mut self, order_by: &'a str) -> Self {
    //     self.order_by_clause = order_by;
    //     self
    // }
}