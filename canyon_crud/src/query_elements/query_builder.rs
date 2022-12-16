use std::fmt::Debug;

use crate::{
    bounds::{FieldIdentifier, FieldValueIdentifier, QueryParameters},
    crud::{CrudOperations, Transaction},
    mapper::RowMapper,
    query_elements::query::Query, Operator,
};

/// Contains the elements that makes part of the formal declaration
/// of the behaviour of the Canyon-SQL QueryBuilder
pub mod ops {
    pub use super::*;

    /// The [`QueryBuilder`] trait is the root of a kind of hierarchy
    /// on more specific [`super::QueryBuilder`], that are:
    /// 
    /// * [`super::SelectQueryBuilder`]
    /// * [`super::UpdateQueryBuilder`]
    /// * [`super::DeleteQueryBuilder`]
    /// 
    /// This trait provides the formal declaration of the behaviour that the
    /// implementors must provide in their public interfaces, groping
    /// the common elements between every element down in that
    /// hierarchy.
    /// 
    /// For example, the [`super::QueryBuilder`] type holds the data
    /// necessary for track the SQL sentece while it's being generated
    /// thought the fluent builder, and provides the behaviour of
    /// the common elements defined in this trait.
    /// 
    /// The more concrete types represents a wrapper over a raw
    /// [`super::QueryBuilder`], offering all the elements declared
    /// in this trait in its public interface, and which implementation
    /// only consists of call the same method on the wrapped
    /// [`super::QueryBuilder`].
    /// 
    /// This allows us to declare in their public interface their
    /// specific operations, like, for example, join operations
    /// on the [`super::SelectQueryBuilder`], and the usage
    /// of the `SET` clause on a [`super::UpdateQueryBuilder`],
    /// without mixing types or convoluting everything into
    /// just one type.
    pub trait QueryBuilder<'a, T> where 
        T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> 
    {
        /// Returns a read-only reference to the underlying SQL sentence,
        /// with the same lifetime as self
        fn read_sql(&'a self) -> &'a str;

        /// Public interface for append the content of an slice to the end of
        /// the underlying SQL sentece.
        /// 
        /// This mutator will allow the user to wire SQL code to the already
        /// generated one
        /// 
        /// * `sql` - The [`&str`] to be wired in the SQL
        fn push_sql(&mut self, sql: &str);

        /// Generates a `WHERE` SQL clause for constraint the query.
        /// 
        /// * `column` - A [`FieldValueIdentifier`] that will provide the target
        /// column name and the value for the filter
        /// * `op` - Any element that implements [`Operator`] for create the comparison
        /// or equality binary operator 
        fn r#where<Z: FieldValueIdentifier<'a, T>>(&mut self, column: Z, op: impl Operator) -> &mut Self
            where T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>;

        /// Generates an `AND` SQL clause for constraint the query.
        /// 
        /// * `column` - A [`FieldValueIdentifier`] that will provide the target
        /// column name and the value for the filter
        /// * `op` - Any element that implements [`Operator`] for create the comparison
        /// or equality binary operator 
        fn and<Z: FieldValueIdentifier<'a, T>>(&mut self, column: Z, op: impl Operator) -> &mut Self;

        /// Generates an `AND` SQL clause for constraint the query that will create
        /// the filter in conjunction with an `IN` operator that will ac
        /// 
        /// * `column` - A [`FieldIdentifier`] that will provide the target
        /// column name for the filter, based on the variant that represents
        /// the field name that maps the targeted column name
        /// * `values` - An array of [`QueryParameters`] with the values to filter
        /// inside the `IN` operator
        fn and_in<Z: FieldIdentifier<T>>(
            &mut self, column: Z, values: &'a [&'a (dyn QueryParameters<'a> + 'a)]
        ) -> &mut Self;

        /// Generates an `OR` SQL clause for constraint the query.
        /// 
        /// * `column` - A [`FieldValueIdentifier`] that will provide the target
        /// column name and the value for the filter
        /// * `op` - Any element that implements [`Operator`] for create the comparison
        /// or equality binary operator 
        fn or<Z: FieldValueIdentifier<'a, T>>(&mut self, column: Z, op: impl Operator) -> &mut Self;

        /// Generates a `ORDER BY` SQL clause for constraint the query.
        /// 
        /// * `order_by` - A [`FieldIdentifier`] that will provide the target
        /// column name
        /// * `desc` - a boolean indicating if the generated `ORDER_BY` must be
        /// in ascending or descending order
        fn order_by<Z: FieldIdentifier<T>>(&mut self, order_by: Z, desc: bool) -> &mut Self;
    }
}

/// Type for construct more complex queries than the classical CRUD ones.
#[derive(Debug)]
pub struct QueryBuilder<'a, T> where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>
{
    query: Query<'a, T>,
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
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>
{
    /// Returns a new instance of the [`QueryBuilder`]
    pub fn new(query: Query<'a, T>, datasource_name: &'a str) -> Self {
        Self { query, datasource_name }
    }

    /// Launches the generated query against the database targeted
    /// by the selected datasource
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

    pub fn and<Z: FieldValueIdentifier<'a, T>>(&mut self, r#and: Z, op: impl Operator) {
        let (column_name, value) = r#and.value();

        let and_ = String::from(" AND ")
            + column_name
            + &op.as_str()
            + "$"
            + &(self.query.params.len() + 1).to_string();

        self.query.sql.push_str(&and_);
        self.query.params.push(value);
    }

    pub fn or<Z: FieldValueIdentifier<'a, T>>(&mut self, r#and: Z, op: impl Operator) {
        let (column_name, value) = r#and.value();

        let and_ = String::from(" OR ")
            + column_name
            + &op.as_str()
            + "$"
            + &(self.query.params.len() + 1).to_string();

        self.query.sql.push_str(&and_);
        self.query.params.push(value);
    }

    pub fn and_in<Z: FieldIdentifier<T>>(
        &mut self, r#and: Z, values: &'a [&'a (dyn QueryParameters<'a> + 'a)]
    ) {
        if values.len() == 0 { return; }

        self.query.sql.push_str(
            &format!(" AND {:?} IN (", r#and.as_str())
        );
        
        let mut counter = 1;
        values.into_iter().for_each(|qp| {
            if values.len() != counter {
                self.query.sql.push_str(
                    &format!("${}, ", self.query.params.len())
                );
                counter +=1;
            } else {
                self.query.sql.push_str(
                    &format!("${}", self.query.params.len())
                );
            }
            self.query.params.push(*qp)
        });

        self.query.sql.push_str(") ");
    }

    #[inline]
    pub fn order_by<Z: FieldIdentifier<T>>(&mut self, order_by: Z, desc: bool) {
        self.query.sql.push_str(
            &(String::from(order_by.as_str()) + if desc { " DESC " } else { "" })
        );
    }

    /// Creates an SQL `SET` clause to especify the columns that must be updated in the sentence
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
            self.query.sql.push_str(column.0.as_str());
            self.query.sql.push_str("=");
            self.query.sql.push_str("'");
            self.query.sql.push_str(&(column.1.to_string() + "'"));
            
            if !(idx + 1) == columns.len() {
                self.query.sql.push_str("', ");
            }

            // TODO Pending to parametrize the SET clause
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
    /// Generates a new public instance of the [`SelectQueryBuilder`]
    pub fn new(table_schema_data: &str, datasource_name: &'a str) -> Self {
        Self { 
            _inner: QueryBuilder::<T>::new(
                Query::new(format!("SELECT * FROM {}", table_schema_data)),
                datasource_name
            )
        }
    }

    /// Adds a *LEFT JOIN* SQL statement to the underlying
    /// [`Query`] holded by the [`QueryBuilder`], where:
    /// 
    /// * `join_table` - The table target of the join operation
    /// * `col1` - The left side of the ON operator for the join 
    /// * `col2` - The right side of the ON operator for the join
    /// 
    /// > Note: The order on the column paramenters is irrelevant 
    pub fn left_join(&mut self, join_table: &str, col1: &str, col2: &str) -> &mut Self {
        self._inner.query.sql.push_str(
            &String::from(
                format!(" LEFT JOIN {join_table} ON {col1} = {col2}")
            )
        );
        self
    }

    /// Adds a *RIGHT JOIN* SQL statement to the underlying
    /// [`Query`] holded by the [`QueryBuilder`], where:
    /// 
    /// * `join_table` - The table target of the join operation
    /// * `col1` - The left side of the ON operator for the join 
    /// * `col2` - The right side of the ON operator for the join
    /// 
    /// > Note: The order on the column paramenters is irrelevant 
    pub fn inner_join(&mut self, join_table: &str, col1: &str, col2: &str) -> &mut Self {
        self._inner.query.sql.push_str(
            &String::from(
                format!(" INNER JOIN {join_table} ON {col1} = {col2}")
            )
        );
        self
    }

    /// Adds a *RIGHT JOIN* SQL statement to the underlying
    /// [`Query`] holded by the [`QueryBuilder`], where:
    /// 
    /// * `join_table` - The table target of the join operation
    /// * `col1` - The left side of the ON operator for the join 
    /// * `col2` - The right side of the ON operator for the join
    /// 
    /// > Note: The order on the column paramenters is irrelevant 
    pub fn right_join(&mut self, join_table: &str, col1: &str, col2: &str) -> &mut Self {
        self._inner.query.sql.push_str(
            &String::from(
                format!(" RIGHT JOIN {join_table} ON {col1} = {col2}")
            )
        );
        self
    }

    /// Adds a *FULL JOIN* SQL statement to the underlying
    /// [`Query`] holded by the [`QueryBuilder`], where:
    /// 
    /// * `join_table` - The table target of the join operation
    /// * `col1` - The left side of the ON operator for the join 
    /// * `col2` - The right side of the ON operator for the join
    /// 
    /// > Note: The order on the column paramenters is irrelevant 
    pub fn full_join(&mut self, join_table: &str, col1: &str, col2: &str) -> &mut Self {
        self._inner.query.sql.push_str(
            &String::from(
                format!(" FULL JOIN {join_table} ON {col1} = {col2}")
            )
        );
        self
    }
}

impl<'a, T> ops::QueryBuilder<'a, T> for SelectQueryBuilder<'a, T> where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> + Send
{
    #[inline]
    fn read_sql(&'a self) -> &'a str {
        self._inner.query.sql.as_str()
    }

    #[inline(always)]
    fn push_sql(&mut self, sql: &str) { 
        self._inner.query.sql.push_str(sql); 
    }

    #[inline]
    fn r#where<Z: FieldValueIdentifier<'a, T>>(&mut self, r#where: Z, op: impl Operator) -> &mut Self {
        self._inner.r#where(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier<'a, T>>(&mut self, column: Z, op: impl Operator) -> &mut Self {
        self._inner.and(column, op);
        self
    }

    #[inline]
    fn and_in<Z: FieldIdentifier<T>>(
        &mut self, r#and: Z, values: &'a [&'a (dyn QueryParameters<'a> + 'a)]
    ) -> &mut Self { self._inner.and_in(and, values); self }

    #[inline]
    fn or<Z: FieldValueIdentifier<'a, T>>(&mut self, column: Z, op: impl Operator) -> &mut Self {
        self._inner.or(column, op);
        self
    }

    #[inline]
    fn order_by<Z: FieldIdentifier<T>>(&mut self, order_by: Z, desc: bool) -> &mut Self {
        self._inner.order_by(order_by, desc);
        self
    }
}


/// Contains the specific database operations of the *UPDATE* SQL statements.
///  
/// * `set` - To construct a new `SET` clause to determine the columns to
/// update with the provided values
#[derive(Debug)]
pub struct UpdateQueryBuilder<'a, T> where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>,
{
    _inner: QueryBuilder<'a, T>,
}

impl<'a, T> UpdateQueryBuilder<'a, T> where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>
{
    /// Generates a new public instance of the [`UpdateQueryBuilder`]
    pub fn new(table_schema_data: &str, datasource_name: &'a str) -> Self {
        Self { 
            _inner: QueryBuilder::<T>::new(
                Query::new(format!("UPDATE {}", table_schema_data)),
                datasource_name
            )
        }
    }

    /// Launches the generated query to the database pointed by the 
    /// selected datasource
    #[inline] pub async fn query(&'a mut self)
        -> Result<Vec<T>, Box<(dyn std::error::Error + Sync + Send + 'static)>>
    { self._inner.query().await }
}

impl<'a, T> ops::QueryBuilder<'a, T> for UpdateQueryBuilder<'a, T> where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> + Send
{
    #[inline]
    fn read_sql(&'a self) -> &'a str {
        self._inner.query.sql.as_str()
    }

    #[inline(always)]
    fn push_sql(&mut self, sql: &str) { 
        self._inner.query.sql.push_str(sql); 
    }

    #[inline]
    fn r#where<Z: FieldValueIdentifier<'a, T>>(&mut self, r#where: Z, op: impl Operator) -> &mut Self {
        self._inner.r#where(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier<'a, T>>(&mut self, column: Z, op: impl Operator) -> &mut Self {
        self._inner.and(column, op);
        self
    }

    #[inline]
    fn and_in<Z: FieldIdentifier<T>>(
        &mut self, r#and: Z, values: &'a [&'a (dyn QueryParameters<'a> + 'a)]
    ) -> &mut Self { self._inner.and_in(and, values); self }

    #[inline]
    fn or<Z: FieldValueIdentifier<'a, T>>(&mut self, column: Z, op: impl Operator) -> &mut Self {
        self._inner.or(column, op);
        self
    }

    #[inline]
    fn order_by<Z: FieldIdentifier<T>>(&mut self, order_by: Z, desc: bool) -> &mut Self {
        self._inner.order_by(order_by, desc);
        self
    }
}


/// Contains the specific database operations associated with the
/// *DELETE* SQL statements.
///  
/// * `set` - To construct a new `SET` clause to determine the columns to
/// update with the provided values
#[derive(Debug)]
pub struct DeleteQueryBuilder<'a, T> where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>,
{
    _inner: QueryBuilder<'a, T>,
}

impl<'a, T> DeleteQueryBuilder<'a, T> where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>
{
    /// Generates a new public instance of the [`DeleteQueryBuilder`]
    pub fn new(table_schema_data: &str, datasource_name: &'a str) -> Self {
        Self { 
            _inner: QueryBuilder::<T>::new(
                Query::new(format!("DELETE FROM {}", table_schema_data)),
                datasource_name
            )
        }
    }

    /// Launches the generated query to the database pointed by the 
    /// selected datasource
    #[inline] pub async fn query(&'a mut self)
        -> Result<Vec<T>, Box<(dyn std::error::Error + Sync + Send + 'static)>>
    { self._inner.query().await }
}

impl<'a, T> ops::QueryBuilder<'a, T> for DeleteQueryBuilder<'a, T> where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> + Send
{
    #[inline]
    fn read_sql(&'a self) -> &'a str {
        self._inner.query.sql.as_str()
    }

    #[inline(always)]
    fn push_sql(&mut self, sql: &str) { 
        self._inner.query.sql.push_str(sql); 
    }

    #[inline]
    fn r#where<Z: FieldValueIdentifier<'a, T>>(&mut self, r#where: Z, op: impl Operator) -> &mut Self {
        self._inner.r#where(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier<'a, T>>(&mut self, column: Z, op: impl Operator) -> &mut Self {
        self._inner.and(column, op);
        self
    }

    #[inline]
    fn and_in<Z: FieldIdentifier<T>>(
        &mut self, r#and: Z, values: &'a [&'a (dyn QueryParameters<'a> + 'a)]
    ) -> &mut Self { self._inner.and_in(and, values); self }

    #[inline]
    fn or<Z: FieldValueIdentifier<'a, T>>(&mut self, column: Z, op: impl Operator) -> &mut Self {
        self._inner.or(column, op);
        self
    }

    #[inline]
    fn order_by<Z: FieldIdentifier<T>>(&mut self, order_by: Z, desc: bool) -> &mut Self {
        self._inner.order_by(order_by, desc);
        self
    }
}