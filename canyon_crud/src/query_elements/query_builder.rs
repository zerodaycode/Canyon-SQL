use std::fmt::Debug;

use tokio_postgres::Error;

use crate::{
    query_elements::query::Query,
    query_elements::operators::Comp,
    crud::{
        Transaction, 
        CrudOperations
    },
    bounds::{
        FieldIdentifier,
        FieldValueIdentifier, 
        InClauseValues
    }, 
    mapper::RowMapper
};


/// Builder for a query while chaining SQL clauses
#[derive(Debug, Clone)]
pub struct QueryBuilder<'a, T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>> {
    query: Query<'a, T>,
    where_clause: String,
    and_clause: String,
    in_clause: &'a[Box<dyn InClauseValues>],
    order_by_clause: String,
    set_clause: String
}
impl<'a, T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>> QueryBuilder<'a, T> {

    // Generates a Query object that contains the necessary data to performn a query
    pub async fn query(&mut self, datasource_name: &str) -> Result<Vec<T>, Error> {
        self.query.sql.retain(|c| !r#";"#.contains(c));

        if self.query.sql.contains("UPDATE") && self.set_clause != "" {
            self.query.sql.push_str(&self.set_clause)
        } else if !self.query.sql.contains("UPDATE") && self.set_clause != "" {
            panic!(
                "'SET' SQL statement only must be used in `T::update_query() associated functions`"
            );
        }
        
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

        self.query.sql.push(';');

        
        let mut unboxed_params = Vec::new();
        for element in self.query.params {
            unboxed_params.push(&**element);
        }

        let result = T::query(
            &self.query.sql[..], 
            &unboxed_params,
            datasource_name
        ).await;

        if let Err(error) = result {
            Err(error)
        } else { Ok(result.ok().unwrap().to_entity::<T>()) }
    }

    pub fn new(query: Query<'a, T>) -> Self {
        Self {
            query: query,
            where_clause: String::new(),
            and_clause: String::new(),
            in_clause: &[],
            order_by_clause: String::new(),
            set_clause: String::new()
        }
    }

    pub fn where_clause<Z: FieldValueIdentifier>(mut self, r#where: Z, comp: Comp) -> Self {
        let values = r#where.value()
            .to_string()
            .split(" ")
            .map( |el| String::from(el))
            .collect::<Vec<String>>();

        let where_ = values.get(0).unwrap().to_string() + 
            &comp.as_string()[..] + "'" +
            values.get(1).unwrap() + "'"; 
        
        self.where_clause.push_str(
            &*(String::from(" WHERE ") + where_.as_str())
        );
        
        self
    } 

    pub fn and_clause<Z: FieldValueIdentifier>(mut self, r#and: Z, comp: Comp) -> Self {
        let values = r#and.value()
            .to_string()
            .split(" ")
            .map( |el| String::from(el))
            .collect::<Vec<String>>();

        let where_ = values.get(0).unwrap().to_string() + 
            &comp.as_string()[..] + "'" +
            values.get(1).unwrap() + "'"; 
        
        self.where_clause.push_str(
            &*(String::from(" AND ") + where_.as_str())
        );

        self
    } 

    pub fn r#in(mut self, in_values: &'a[Box<dyn InClauseValues>]) -> Self {
        self.in_clause = in_values;
        self
    } 

    pub fn order_by<Z: FieldIdentifier>(mut self, order_by: Z, desc: bool) -> Self {
        let desc = if desc { String::from(" DESC ") 
            } else { "".to_owned() };

        self.order_by_clause.push_str(
            &*(
                String::from(" ORDER BY ") + 
                order_by.field_name_as_str().as_str() + 
                &desc
            )
        );
        self
    }

    /// The SQL `SET` clause to especify the columns that must be updated in the sentence
    pub fn set_clause<Z, S>(mut self, columns: &'a[(Z, S)]) -> Self 
        where Z: FieldIdentifier + Clone, S: ToString 
    {
        if columns.len() == 0 {
            return self;
        } else if columns.len() > 0 {
            self.set_clause.push_str(" SET ")
        }

        for (idx, column) in columns.iter().enumerate() {
            if idx + 1 == columns.len() {
                self.set_clause.push_str(
                    &(column.0.clone().field_name_as_str().to_owned() + "=" + "'" + column.1.to_string().as_str() + "'")
                )
            } else {
                self.set_clause.push_str(
                    &(column.0.clone().field_name_as_str().to_owned() + "=" + "'" + column.1.to_string().as_str() + "', ")
                )
            }
        }
        self
    }
}