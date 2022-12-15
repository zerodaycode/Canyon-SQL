use std::fmt::Debug;

use async_trait::async_trait;

use crate::{
    bounds::{FieldIdentifier, FieldValueIdentifier, QueryParameters},
    crud::{CrudOperations, Transaction},
    mapper::RowMapper,
    query_elements::query::Query, Operator,
};

#[async_trait]
pub trait BaseQueryBuilder<'a, T> where 
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> 
{
    async fn query(&'a mut self)
        -> Result<Vec<T>, Box<(dyn std::error::Error + Sync + Send + 'static)>>;

    fn r#where<Z: FieldValueIdentifier<'a, T>>(&mut self, r#where: Z, op: impl Operator) -> &mut Self
        where T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>;
}

/// Type for construct more complex queries than the classical CRUD ones.
#[derive(Debug)]
pub struct QueryBuilder<'a, T> where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>
{
    pub query: Query<'a, T>,  // TODO Decouple Query from Querybuilder
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
    /// Returns an only-read reference to the underlying SQL sentence
    pub fn get_sql(&'a self) -> &'a str {
        self.query.sql.as_str()
    }
    /// Public interface for append the content of an slice to the end of
    /// the underlying SQL sentece
    pub fn push_sql(&mut self, sql: &str) { self.query.sql.push_str(sql); }

    /// Generates a Query object that contains the necessary data to performn a query
    #[allow(clippy::question_mark)]
    pub async fn query(&'a mut self)
        -> Result<Vec<T>, Box<(dyn std::error::Error + Sync + Send + 'static)>>
    {
        // Close the query, we are ready to go
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
            datasource_name
        }
    }

    pub fn r#where<Z: FieldValueIdentifier<'a, T>>(&mut self, r#where: Z, op: impl Operator) {
        let (column_name, value) = r#where.value();

        let where_ = String::from(" WHERE ") 
            + column_name
            + &op.as_str()
            + "$"
            + &(self.query.params.len() + 1).to_string();

        self.query.sql.push_str(&where_);
        self.query.params.push(value);
    }

    pub fn and_clause<Z: FieldValueIdentifier<'a, T>>(mut self, r#and: Z, op: impl Operator) -> Self {
        let (column_name, value) = r#and.value();

        let and_ = String::from(" AND ")
            + column_name
            + &op.as_str()
            + "$"
            + &(self.query.params.len() + 1).to_string();

        self.query.sql.push_str(&and_);
        self.query.params.push(value);
        self
    }

    pub fn or_clause<Z: FieldValueIdentifier<'a, T>>(mut self, r#and: Z, op: impl Operator) -> Self {
        let (column_name, value) = r#and.value();

        let and_ = String::from(" OR ")
            + column_name
            + &op.as_str()
            + "$"
            + &(self.query.params.len() + 1).to_string();

        self.query.sql.push_str(&and_);
        self.query.params.push(value);
        self
    }

    pub fn and<Z: FieldIdentifier<T>>(mut self, column: Z) -> Self {
        self.query.sql.push_str(&(String::from(" AND ") + &column.as_str()));
        self
    }

    #[inline]
    pub fn or<Z: FieldIdentifier<T>>(mut self, column: Z) -> Self {
        self.query.sql.push_str(
            &(String::from(" OR ") + column.as_str())
        );
        self
    }

    pub fn r#in(mut self, in_values: &'a [&'a (dyn QueryParameters<'a> + 'a)]) -> Self {
        self.query.sql.push_str("(");
        
        in_values.into_iter().for_each(
            |qp| {
                self.query.sql.push_str(
                    &format!("${}", self.query.params.len())
                );
                self.query.params.push(*qp)
            }
        );

        self.query.sql.push_str(") ");
        self
    }

    #[inline]
    pub fn order_by<Z: FieldIdentifier<T>>(mut self, order_by: Z, desc: bool) -> Self {
        self.query.sql.push_str(
            &(order_by.field_name_as_str() + if desc { " DESC " } else { "" })
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
            _ => self.query.sql.push_str(" SET "),
        }

        for (idx, column) in columns.iter().enumerate() {
            if idx + 1 == columns.len() {
                self.query.sql.push_str(
                    &(column.0.clone().field_name_as_str().to_owned()
                        + "="
                        + "'"
                        + column.1.to_string().as_str()
                        + "'"
                    ),
                )
            } else {
                self.query.sql.push_str(
                    &(column.0.clone().field_name_as_str().to_owned()
                        + "="
                        + "'"
                        + column.1.to_string().as_str()
                        + "', "
                    ),
                )
            }
        }
        self
    }
}

#[derive(Debug)]
pub struct SelectQueryBuilder<'a, T> where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>,
{
    _inner: QueryBuilder<'a, T>,
}
impl<'a, T> SelectQueryBuilder<'a, T> where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>
{
    pub fn new(table_schema_data: &str) -> Self {
        Self { 
            _inner: Query::generate(format!("SELECT * FROM {}", table_schema_data), "")
        } 
    }
    pub fn left_join(&mut self, join_table: &str, col1: &str, col2: &str) -> &mut Self {
        self._inner.query.sql.push_str(
            &String::from(
                format!("LEFT JOIN {join_table} ON {col1} = {col2} ")
            )
        );
        self
    }
    pub fn inner_join(&mut self, join_table: &str, col1: &str, col2: &str) -> &mut Self {
        self._inner.query.sql.push_str(
            &String::from(
                format!("INNER JOIN {join_table} ON {col1} = {col2} ")
            )
        );
        self
    }
    pub fn right_join(&mut self, join_table: &str, col1: &str, col2: &str) -> &mut Self {
        self._inner.query.sql.push_str(
            &String::from(
                format!("RIGHT JOIN {join_table} ON {col1} = {col2} ")
            )
        );
        self
    }
    pub fn full_join(&mut self, join_table: &str, col1: &str, col2: &str) -> &mut Self {
        self._inner.query.sql.push_str(
            &String::from(
                format!("FULL JOIN {join_table} ON {col1} = {col2} ")
            )
        );
        self
    }
}

#[async_trait]
impl<'a, T> BaseQueryBuilder<'a, T> for SelectQueryBuilder<'a, T>
where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> + Send
{
    #[inline]
    async fn query(&'a mut self) -> Result<Vec<T>, Box<(dyn std::error::Error + Sync + Send + 'static)>> {
        self._inner.query().await
    }

    #[inline]
    fn r#where<Z: FieldValueIdentifier<'a, T>>(&mut self, r#where: Z, op: impl Operator) -> &mut Self {
        self._inner.r#where(r#where, op);
        self
    }
}