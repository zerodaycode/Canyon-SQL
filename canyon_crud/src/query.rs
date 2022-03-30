use tokio_postgres::types::ToSql;
use std::{fmt::Debug, marker::PhantomData};

use crate::{crud::{Transaction, CrudOperations}, result::DatabaseResult};

/// Holds a mut sql sentence
#[derive(Debug, Clone)]
pub struct Query<'a, T: Debug + CrudOperations<T> + Transaction<T>> {
    sql: String,
    params: &'a[Box<dyn ToSql + Sync>],
    marker: PhantomData<T>
}

impl<'a, T> Query<'a, T> where T: Debug + CrudOperations<T> + Transaction<T> {
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
#[derive(Debug, Clone)]
pub struct QueryBuilder<'a, T: Debug + CrudOperations<T> + Transaction<T>> {
    query: Query<'a, T>,
    where_clause: String,
    and_clause: String,
    // in_clause: &'a[Box<dyn ToSql>],
    order_by_clause: String
}
impl<'a, T: Debug + CrudOperations<T> + Transaction<T>> QueryBuilder<'a, T> {

    // Generates a Query object that contains the necessary data to performn a query
    pub async fn query(&mut self) -> DatabaseResult<T> {
        self.query.sql.retain(|c| !r#";"#.contains(c));
        
        if self.where_clause != "" {
            self.query.sql.push_str(&self.where_clause)
        }
        if self.and_clause != "" {
            self.query.sql.push_str(&self.and_clause)
        }
        if self.order_by_clause != "" {
            self.query.sql.push_str(&self.order_by_clause)
        }
        // ... rest of statements

        self.query.sql.push(';');

        
        let mut unboxed_params = Vec::new();
        for element in self.query.params {
            unboxed_params.push(&**element);
        }

        println!("Executing query: {:?}", &self.query.sql);

        T::query(&self.query.sql[..], &unboxed_params).await
    }

    pub fn new(query: Query<'a, T>) -> Self {
        Self {
            query: query,
            where_clause: String::new(),
            and_clause: String::new(),
            // in_clause: &[],
            order_by_clause: String::new()
        }
    }

    pub fn where_clause(mut self, r#where: &'a str) -> Self {
        self.where_clause.push_str(&*(String::from(" WHERE ") + r#where));
        self
    } 

    pub fn and_clause(mut self, r#and: &'a str) -> Self {
        self.and_clause.push_str(&*(String::from(" AND ") + r#and));
        self
    } 

    // pub fn r#in(mut self, in_values: &'a[Box<dyn ToSql>]) -> Self {
    //     self.in_clause = in_values;
    //     self
    // } 

    pub fn order_by(mut self, order_by: &'a str) -> Self {
        self.order_by_clause.push_str(&*(String::from(" ORDER BY ") + order_by));
        self
    }
}