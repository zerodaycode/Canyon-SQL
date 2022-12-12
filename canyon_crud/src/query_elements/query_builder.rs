use std::fmt::Debug;

use crate::{
    bounds::{FieldIdentifier, FieldValueIdentifier, QueryParameters},
    crud::{CrudOperations, Transaction},
    mapper::RowMapper,
    query_elements::operators::Comp,
    query_elements::query::Query,
};

/// Builder for a query while chaining SQL clauses
// #[derive(Clone)]
pub struct QueryBuilder<'a, T>
where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>,
{
    query: Query<'a, T>,
    where_clause: String,
    and_clause: String,
    in_clause: String,
    order_by_clause: String,
    set_clause: String,
    datasource_name: &'a str
}

unsafe impl<'a, T> Send for QueryBuilder<'a, T> 
where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> {}

unsafe impl<'a, T> Sync for QueryBuilder<'a, T> 
where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> {}


impl<'a, T> QueryBuilder<'a, T>
where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>,
{
    /// Generates a Query object that contains the necessary data to performn a query
    #[allow(clippy::question_mark)]
    pub async fn query(
        &'a mut self,
    ) -> Result<Vec<T>, Box<(dyn std::error::Error + Sync + Send + 'static)>> {
        self.query.sql.retain(|c| !r#";"#.contains(c));

        if self.query.sql.contains("UPDATE") && !self.set_clause.is_empty() {
            self.query.sql.push_str(&self.set_clause)
        } else if !self.query.sql.contains("UPDATE") && !self.set_clause.is_empty() {
            panic!(
                "'SET' SQL statement only must be used in `T::update_query() associated functions`"
            );
        }

        if self.where_clause.len() > 7 {
            self.query.sql.push_str(&self.where_clause)
        }

        if self.and_clause.len() > 5 {
            self.query.sql.push_str(&self.and_clause)
        }

        if self.in_clause.len() > 4 {
            self.query.sql.push_str(&self.in_clause)
        }

        if self.order_by_clause.len() > 10 {
            self.query.sql.push_str(&self.order_by_clause)
        }

        self.query.sql.push(';');

        let result = T::query(
            self.query.sql.clone(),
            self.query.params.iter().map(|arg| *arg).collect::<Vec<&dyn QueryParameters>>(),
            self.datasource_name,
        ).await;

        if let Err(error) = result {
            Err(error)
        } else {
            Ok(result.ok().unwrap().get_entities::<T>())
        }
    }

    pub fn new(query: Query<'a, T>, datasource_name: &'a str) -> Self {
        Self {
            query,
            where_clause: String::from(" WHERE "),
            and_clause: String::from(" AND "),
            in_clause: String::from(" IN "),
            order_by_clause: String::from(" ORDER BY "),
            set_clause: String::new(),
            datasource_name
        }
    }

    pub fn r#where<Z: FieldValueIdentifier<'a, T>>(mut self, r#where: Z, comp: Comp) -> Self {
        let (column_name, value) = r#where.value();

        let where_ = column_name.to_string()
            + &comp.as_string()[..]
            + "$"
            + &(self.query.params.len() + 1).to_string();

        self.where_clause.push_str(&where_);
        self.query.params.push(value);
        self
    }

    pub fn and<Z: FieldValueIdentifier<'a, T>>(mut self, r#and: Z, comp: Comp) -> Self {
        let (column_name, value) = r#and.value();

        let and_ = column_name.to_string()
            + &comp.as_string()[..]
            + "$"
            + &(self.query.params.len() + 1).to_string();

        self.and_clause.push_str(&and_);
        self.query.params.push(value);
        self
    }

    pub fn r#in(mut self, in_values: &'a [&'a (dyn QueryParameters<'a> + 'a)]) -> Self {
        self.in_clause.push_str("(");
        
        in_values.into_iter().for_each(
            |qp| {
                self.in_clause.push_str(
                    &format!("${}",self.query.params.len())
                );
                self.query.params.push(*qp)
            }
        );

        self.in_clause.push_str(") ");
        self
    }

    pub fn order_by<Z: FieldIdentifier<T>>(mut self, order_by: Z, desc: bool) -> Self {
        let desc = if desc {
            String::from(" DESC ")
        } else {
            "".to_owned()
        };

        self.order_by_clause.push_str(
            &(order_by.field_name_as_str() + &desc),
        );
        self
    }

    /// The SQL `SET` clause to especify the columns that must be updated in the sentence
    pub fn set<Z, S>(mut self, columns: &'a [(Z, S)]) -> Self
    where
        Z: FieldIdentifier<T> + Clone,
        S: ToString,
    {
        match columns.len() {
            0 => return self,
            _ => self.set_clause.push_str(" SET "),
        }

        for (idx, column) in columns.iter().enumerate() {
            if idx + 1 == columns.len() {
                self.set_clause.push_str(
                    &(column.0.clone().field_name_as_str().to_owned()
                        + "="
                        + "'"
                        + column.1.to_string().as_str()
                        + "'"),
                )
            } else {
                self.set_clause.push_str(
                    &(column.0.clone().field_name_as_str().to_owned()
                        + "="
                        + "'"
                        + column.1.to_string().as_str()
                        + "', "),
                )
            }
        }
        self
    }
}
