use tokio_postgres::types::ToSql;

use crate::{crud::Transaction, result::DatabaseResult};

/// Holds a mut sql sentence
#[derive(Debug)]
pub struct Query<'a> {
    sql: String,
    params: &'a[Box<dyn ToSql>]
}

impl <'a> Transaction<Self> for Query<'a> {}

impl<'a> Query<'a> {
    pub fn new(sql: String, params: &'a[Box<dyn ToSql>]) -> QueryBuilder<'a> {
        let self_ = Self {
            sql: sql,
            params: params
        };
        QueryBuilder::new(self_)
    }
}

/// Builder for a query while chaining SQL clauses
#[derive(Debug)]
pub struct QueryBuilder<'a> {
    query: Query<'a>,
    where_clause: &'a str,
    // and_clause: &'a str,
    // in_clause: &'a[Box<dyn ToSql>],
    // order_by_clause: &'a str
}
impl<'a> QueryBuilder<'a> {

    // Generates a Query object that contains the necessary data to performn a query
    pub async fn query(mut self) -> DatabaseResult<Query<'a>> {
        if self.where_clause != "" {
            self.query.sql.push_str(self.where_clause)
        }
        Query::query(
            self.query.sql.as_ref(), 
            self.query.params
                .iter()
                .map( |boxed| boxed.try_into() )
                // Expects dyn ToSql + Sync
        ).await
    }

    pub fn new(query: Query<'a>) -> Self {
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