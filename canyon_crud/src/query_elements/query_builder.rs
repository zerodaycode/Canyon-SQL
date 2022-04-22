use std::fmt::Debug;

use tokio_postgres::types::ToSql;

use crate::{
    query_elements::query::Query,
    query_elements::operators::Comp,
    crud::{Transaction, CrudOperations},
    bounds::FieldIdentifier, mapper::RowMapper
};


/// Builder for a query while chaining SQL clauses
#[derive(Debug, Clone)]
pub struct QueryBuilder<'a, T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>> {
    query: Query<'a, T>,
    where_clause: String,
    and_clause: String,
    in_clause: &'a[Box<dyn InClauseValues>],
    order_by_clause: String
}
impl<'a, T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>> QueryBuilder<'a, T> {

    // Generates a Query object that contains the necessary data to performn a query
    pub async fn query(&mut self) -> Vec<T> {
        self.query.sql.retain(|c| !r#";"#.contains(c));
        
        if self.where_clause != "" {
            self.query.sql.push_str(&self.where_clause)
        }
        if self.and_clause != "" {
            self.query.sql.push_str(&self.and_clause)
        }
        if self.in_clause.is_empty() {
            for value in self.in_clause {
                self.query.sql.push_str(&value.to_string())
            }
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

        T::query(&self.query.sql[..], &unboxed_params).await.as_response::<T>()
    }

    pub fn new(query: Query<'a, T>) -> Self {
        Self {
            query: query,
            where_clause: String::new(),
            and_clause: String::new(),
            in_clause: &[],
            order_by_clause: String::new()
        }
    }

    pub fn where_clause<Z: FieldIdentifier>(mut self, r#where: Z, comp: Comp) -> Self {
        let values = r#where.value()
            .to_string()
            .split(" ")
            .map( |el| String::from(el))
            .collect::<Vec<String>>();

        let where_ = values.get(0).unwrap().to_string() + 
            &comp.as_string()[..] + 
            values.get(1).unwrap(); 
        
        self.where_clause.push_str(
            &*(String::from(" WHERE ") + where_.as_str())
        );
        
        self
    } 

    pub fn and_clause(mut self, r#and: &'a str) -> Self {
        self.and_clause.push_str(&*(String::from(" AND ") + r#and));
        self
    } 

    pub fn r#in(mut self, in_values: &'a[Box<dyn InClauseValues>]) -> Self {
        self.in_clause = in_values;
        self
    } 

    pub fn order_by(mut self, order_by: &'a str) -> Self {
        self.order_by_clause.push_str(&*(String::from(" ORDER BY ") + order_by));
        self
    }
}

/// To define trait objects that helps to relates the necessary bounds n the 'in_clause`
pub trait InClauseValues: ToSql + ToString {}