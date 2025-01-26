use crate::{
    bounds::{FieldIdentifier, FieldValueIdentifier},
    crud::CrudOperations,
    query_elements::query::Query,
    Operator,
};
use canyon_core::connection::{database_type::DatabaseType, get_database_config, DATASOURCES};
use canyon_core::query::TransactionInput;
use canyon_core::{mapper::RowMapper, query::Transaction, query_parameters::QueryParameter};
use std::fmt::Debug;
use std::marker::PhantomData;

/// Contains the elements that makes part of the formal declaration
/// of the behaviour of the Canyon-SQL QueryBuilder
pub mod ops {
    use canyon_core::{mapper::RowMapper, query::Transaction, query_parameters::QueryParameter};

    use crate::crud::CrudOperations;

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
    /// necessary for track the SQL sentence while it's being generated
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
    pub trait QueryBuilder<'a, T>
    where
        T: CrudOperations<T> + Transaction<T> + RowMapper<T>,
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
        fn push_sql(self, sql: &str);

        /// Generates a `WHERE` SQL clause for constraint the query.
        ///
        /// * `column` - A [`FieldValueIdentifier`] that will provide the target
        ///     column name and the value for the filter
        /// * `op` - Any element that implements [`Operator`] for create the comparison
        ///     or equality binary operator
        fn r#where<Z: FieldValueIdentifier<'a, T>>(
            self,
            column: Z,
            op: impl Operator,
        ) -> Self
        where
            T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T>;

        /// Generates an `AND` SQL clause for constraint the query.
        ///
        /// * `column` - A [`FieldValueIdentifier`] that will provide the target
        ///     column name and the value for the filter
        /// * `op` - Any element that implements [`Operator`] for create the comparison
        ///     or equality binary operator
        fn and<Z: FieldValueIdentifier<'a, T>>(
            self,
            column: Z,
            op: impl Operator,
        ) -> Self;

        /// Generates an `AND` SQL clause for constraint the query that will create
        /// the filter in conjunction with an `IN` operator that will ac
        ///
        /// * `column` - A [`FieldIdentifier`] that will provide the target
        ///     column name for the filter, based on the variant that represents
        ///     the field name that maps the targeted column name
        /// * `values` - An array of [`QueryParameter`] with the values to filter
        ///     inside the `IN` operator
        fn and_values_in<Z, Q>(self, column: Z, values: &'a [Q]) -> Self
        where
            Z: FieldIdentifier<T>,
            Q: QueryParameter<'a>;

        /// Generates an `OR` SQL clause for constraint the query that will create
        /// the filter in conjunction with an `IN` operator that will ac
        ///
        /// * `column` - A [`FieldIdentifier`] that will provide the target
        ///     column name for the filter, based on the variant that represents
        ///     the field name that maps the targeted column name
        /// * `values` - An array of [`QueryParameter`] with the values to filter
        ///     inside the `IN` operator
        fn or_values_in<Z, Q>(self, r#or: Z, values: &'a [Q]) -> Self
        where
            Z: FieldIdentifier<T>,
            Q: QueryParameter<'a>;

        /// Generates an `OR` SQL clause for constraint the query.
        ///
        /// * `column` - A [`FieldValueIdentifier`] that will provide the target
        ///     column name and the value for the filter
        /// * `op` - Any element that implements [`Operator`] for create the comparison
        ///     or equality binary operator
        fn or<Z: FieldValueIdentifier<'a, T>>(self, column: Z, op: impl Operator)
            -> Self;

        /// Generates a `ORDER BY` SQL clause for constraint the query.
        ///
        /// * `order_by` - A [`FieldIdentifier`] that will provide the target  column name
        /// * `desc` - a boolean indicating if the generated `ORDER_BY` must be in ascending or descending order
        fn order_by<Z: FieldIdentifier<T>>(self, order_by: Z, desc: bool) -> Self;
    }
}

/// Type for construct more complex queries than the classical CRUD ones.
pub struct QueryBuilder<'a, T, I>
where
    T: CrudOperations<T> + Transaction<T> + RowMapper<T>,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
{
    query: Query<'a>,
    input: I,
    datasource_type: DatabaseType,
    pd: PhantomData<T>, // TODO: provisional while reworking the bounds
}

unsafe impl<'a, T, I> Send for QueryBuilder<'a, T, I>
where
    T: CrudOperations<T> + Transaction<T> + RowMapper<T>,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
{
}
unsafe impl<'a, T, I> Sync for QueryBuilder<'a, T, I>
where
    T: CrudOperations<T> + Transaction<T> + RowMapper<T>,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
{
}

impl<'a, T, I> QueryBuilder<'a, T, I>
where
    T: CrudOperations<T> + Transaction<T> + RowMapper<T>,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
    TransactionInput<'a>: From<&'a I>,
{
    /// Returns a new instance of the [`QueryBuilder`]
    pub fn new(query: Query<'a>, input: I) -> Self {
        Self {
            query,
            input,
            datasource_type: todo!("The from type on the querybuilder"),
            // DatabaseType::from(
            //     &get_database_config(input, &DATASOURCES).auth,
            // ),
            pd: Default::default(),
        }
    }

    /// Launches the generated query against the database targeted
    /// by the selected datasource
    pub async fn query(
        mut self,
    ) -> Result<Vec<T>, Box<(dyn std::error::Error + Sync + Send + 'a)>> {
        self.query.sql.push(';');

        Ok(T::query(
            self.query.sql.clone(),
            self.query.params.to_vec(),
            self.input,
        )
        .await?
        .into_results::<T>())
    }

    pub fn r#where<Z: FieldValueIdentifier<'a, T>>(&mut self, r#where: Z, op: impl Operator) {
        let (column_name, value) = r#where.value();

        let where_ = String::from(" WHERE ")
            + column_name
            + &op.as_str(self.query.params.len() + 1, &self.datasource_type);

        self.query.sql.push_str(&where_);
        self.query.params.push(value);
    }

    pub fn and<Z: FieldValueIdentifier<'a, T>>(&mut self, r#and: Z, op: impl Operator) {
        let (column_name, value) = r#and.value();

        let and_ = String::from(" AND ")
            + column_name
            + &op.as_str(self.query.params.len() + 1, &self.datasource_type);

        self.query.sql.push_str(&and_);
        self.query.params.push(value);
    }

    pub fn or<Z: FieldValueIdentifier<'a, T>>(&mut self, r#and: Z, op: impl Operator) {
        let (column_name, value) = r#and.value();

        let and_ = String::from(" OR ")
            + column_name
            + &op.as_str(self.query.params.len() + 1, &self.datasource_type);

        self.query.sql.push_str(&and_);
        self.query.params.push(value);
    }

    pub fn and_values_in<Z, Q>(&mut self, r#and: Z, values: &'a [Q])
    where
        Z: FieldIdentifier<T>,
        Q: QueryParameter<'a>,
    {
        if values.is_empty() {
            return;
        }

        self.query
            .sql
            .push_str(&format!(" AND {} IN (", r#and.as_str()));

        let mut counter = 1;
        values.iter().for_each(|qp| {
            if values.len() != counter {
                self.query
                    .sql
                    .push_str(&format!("${}, ", self.query.params.len()));
                counter += 1;
            } else {
                self.query
                    .sql
                    .push_str(&format!("${}", self.query.params.len()));
            }
            self.query.params.push(qp)
        });

        self.query.sql.push(')')
    }

    fn or_values_in<Z, Q>(&mut self, r#or: Z, values: &'a [Q])
    where
        Z: FieldIdentifier<T>,
        Q: QueryParameter<'a>,
    {
        if values.is_empty() {
            return;
        }

        self.query
            .sql
            .push_str(&format!(" OR {} IN (", r#or.as_str()));

        let mut counter = 1;
        values.iter().for_each(|qp| {
            if values.len() != counter {
                self.query
                    .sql
                    .push_str(&format!("${}, ", self.query.params.len()));
                counter += 1;
            } else {
                self.query
                    .sql
                    .push_str(&format!("${}", self.query.params.len()));
            }
            self.query.params.push(qp)
        });

        self.query.sql.push(')')
    }

    #[inline]
    pub fn order_by<Z: FieldIdentifier<T>>(&mut self, order_by: Z, desc: bool) {
        self.query.sql.push_str(
            &(format!(
                " ORDER BY {}{}",
                order_by.as_str(),
                if desc { " DESC " } else { "" }
            )),
        );
    }
}

pub struct SelectQueryBuilder<'a, T, I>
where
    T: CrudOperations<T> + Transaction<T> + RowMapper<T>,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
    TransactionInput<'a>: From<&'a I>,
{
    _inner: QueryBuilder<'a, T, I>,
}

impl<'a, T, I> SelectQueryBuilder<'a, T, I>
where
    T: CrudOperations<T> + Transaction<T> + RowMapper<T>,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
    TransactionInput<'a>: From<&'a I>,
{
    /// Generates a new public instance of the [`SelectQueryBuilder`]
    pub fn new(table_schema_data: &str, input: I) -> Self {
        Self {
            _inner: QueryBuilder::<T, I>::new(
                Query::new(format!("SELECT * FROM {table_schema_data}")),
                input,
            ),
        }
    }

    /// Launches the generated query to the database pointed by the
    /// selected datasource
    #[inline]
    pub async fn query(self) -> Result<Vec<T>, Box<(dyn std::error::Error + Sync + Send + 'a)>> {
        self._inner.query().await
    }

    /// Adds a *LEFT JOIN* SQL statement to the underlying
    /// [`Query`] held by the [`QueryBuilder`], where:
    ///
    /// * `join_table` - The table target of the join operation
    /// * `col1` - The left side of the ON operator for the join
    /// * `col2` - The right side of the ON operator for the join
    ///
    /// > Note: The order on the column parameters is irrelevant
    pub fn left_join(mut self, join_table: &str, col1: &str, col2: &str) -> Self {
        self._inner
            .query
            .sql
            .push_str(&format!(" LEFT JOIN {join_table} ON {col1} = {col2}"));
        self
    }

    /// Adds a *INNER JOIN* SQL statement to the underlying
    /// [`Query`] held by the [`QueryBuilder`], where:
    ///
    /// * `join_table` - The table target of the join operation
    /// * `col1` - The left side of the ON operator for the join
    /// * `col2` - The right side of the ON operator for the join
    ///
    /// > Note: The order on the column parameters is irrelevant
    pub fn inner_join(mut self, join_table: &str, col1: &str, col2: &str) -> Self {
        self._inner
            .query
            .sql
            .push_str(&format!(" INNER JOIN {join_table} ON {col1} = {col2}"));
        self
    }

    /// Adds a *RIGHT JOIN* SQL statement to the underlying
    /// [`Query`] held by the [`QueryBuilder`], where:
    ///
    /// * `join_table` - The table target of the join operation
    /// * `col1` - The left side of the ON operator for the join
    /// * `col2` - The right side of the ON operator for the join
    ///
    /// > Note: The order on the column parameters is irrelevant
    pub fn right_join(mut self, join_table: &str, col1: &str, col2: &str) -> Self {
        self._inner
            .query
            .sql
            .push_str(&format!(" RIGHT JOIN {join_table} ON {col1} = {col2}"));
        self
    }

    /// Adds a *FULL JOIN* SQL statement to the underlying
    /// [`Query`] held by the [`QueryBuilder`], where:
    ///
    /// * `join_table` - The table target of the join operation
    /// * `col1` - The left side of the ON operator for the join
    /// * `col2` - The right side of the ON operator for the join
    ///
    /// > Note: The order on the column parameters is irrelevant
    pub fn full_join(mut self, join_table: &str, col1: &str, col2: &str) -> Self {
        self._inner
            .query
            .sql
            .push_str(&format!(" FULL JOIN {join_table} ON {col1} = {col2}"));
        self
    }
}

impl<'a, T, I> ops::QueryBuilder<'a, T> for SelectQueryBuilder<'a, T, I>
where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> + Send,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
    TransactionInput<'a>: From<&'a I>,
{
    #[inline]
    fn read_sql(&'a self) -> &'a str {
        self._inner.query.sql.as_str()
    }

    #[inline(always)]
    fn push_sql(mut self, sql: &str) {
        self._inner.query.sql.push_str(sql);
    }

    #[inline]
    fn r#where<Z: FieldValueIdentifier<'a, T>>(
        mut self,
        r#where: Z,
        op: impl Operator,
    ) -> Self {
        self._inner.r#where(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier<'a, T>>(mut self, column: Z, op: impl Operator) -> Self {
        self._inner.and(column, op);
        self
    }

    #[inline]
    fn and_values_in<Z, Q>(mut self, r#and: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier<T>,
        Q: QueryParameter<'a>,
    {
        self._inner.and_values_in(and, values);
        self
    }

    #[inline]
    fn or_values_in<Z, Q>(mut self, r#and: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier<T>,
        Q: QueryParameter<'a>,
    {
        self._inner.or_values_in(and, values);
        self
    }

    #[inline]
    fn or<Z: FieldValueIdentifier<'a, T>>(mut self, column: Z, op: impl Operator) -> Self {
        self._inner.or(column, op);
        self
    }

    #[inline]
    fn order_by<Z: FieldIdentifier<T>>(mut self, order_by: Z, desc: bool) -> Self {
        self._inner.order_by(order_by, desc);
        self
    }
}

/// Contains the specific database operations of the *UPDATE* SQL statements.
///
/// * `set` - To construct a new `SET` clause to determine the columns to
///     update with the provided values
pub struct UpdateQueryBuilder<'a, T, I>
where
    T: CrudOperations<T> + Transaction<T> + RowMapper<T>,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
    TransactionInput<'a>: From<&'a I>,
{
    _inner: QueryBuilder<'a, T, I>,
}

impl<'a, T, I> UpdateQueryBuilder<'a, T, I>
where
    T: CrudOperations<T> + Transaction<T> + RowMapper<T>,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
    TransactionInput<'a>: From<&'a I>,
{
    /// Generates a new public instance of the [`UpdateQueryBuilder`]
    pub fn new(table_schema_data: &str, input: I) -> Self {
        Self {
            _inner: QueryBuilder::<T, I>::new(
                Query::new(format!("UPDATE {table_schema_data}")),
                input,
            ),
        }
    }

    /// Launches the generated query to the database pointed by the
    /// selected datasource
    #[inline]
    pub async fn query(
        self,
    ) -> Result<Vec<T>, Box<(dyn std::error::Error + Sync + Send + 'a)>> {
        self._inner.query().await
    }

    /// Creates an SQL `SET` clause to specify the columns that must be updated in the sentence
    pub fn set<Z, Q>(mut self, columns: &'a [(Z, Q)]) -> Self
    where
        Z: FieldIdentifier<T> + Clone,
        Q: QueryParameter<'a>,
    {
        if columns.is_empty() {
            return self;
        }
        if self._inner.query.sql.contains("SET") {
            panic!(
                // TODO: this should return an Err and not panic!
                "\n{}",
                String::from("\t[PANIC!] - Don't use chained calls of the .set(...) method. ")
                    + "\n\tPass all the values in a unique call within the 'columns' "
                    + "array of tuples parameter\n"
            )
        }

        let mut set_clause = String::new();
        set_clause.push_str(" SET ");

        for (idx, column) in columns.iter().enumerate() {
            set_clause.push_str(&format!(
                "{} = ${}",
                column.0.as_str(),
                self._inner.query.params.len() + 1
            ));

            if idx < columns.len() - 1 {
                set_clause.push_str(", ");
            }
            self._inner.query.params.push(&column.1);
        }

        self._inner.query.sql.push_str(&set_clause);
        self
    }
}

impl<'a, T, I> ops::QueryBuilder<'a, T> for UpdateQueryBuilder<'a, T, I>
where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> + Send,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
    TransactionInput<'a>: From<&'a I>,
{
    #[inline]
    fn read_sql(&'a self) -> &'a str {
        self._inner.query.sql.as_str()
    }

    #[inline(always)]
    fn push_sql(mut self, sql: &str) {
        self._inner.query.sql.push_str(sql);
    }

    #[inline]
    fn r#where<Z: FieldValueIdentifier<'a, T>>(
        mut self,
        r#where: Z,
        op: impl Operator,
    ) -> Self {
        self._inner.r#where(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier<'a, T>>(mut self, column: Z, op: impl Operator) -> Self {
        self._inner.and(column, op);
        self
    }

    #[inline]
    fn and_values_in<Z, Q>(mut self, r#and: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier<T>,
        Q: QueryParameter<'a>,
    {
        self._inner.and_values_in(and, values);
        self
    }

    #[inline]
    fn or_values_in<Z, Q>(mut self, r#or: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier<T>,
        Q: QueryParameter<'a>,
    {
        self._inner.or_values_in(or, values);
        self
    }

    #[inline]
    fn or<Z: FieldValueIdentifier<'a, T>>(mut self, column: Z, op: impl Operator) -> Self {
        self._inner.or(column, op);
        self
    }

    #[inline]
    fn order_by<Z: FieldIdentifier<T>>(mut self, order_by: Z, desc: bool) -> Self {
        self._inner.order_by(order_by, desc);
        self
    }
}

/// Contains the specific database operations associated with the
/// *DELETE* SQL statements.
///
/// * `set` - To construct a new `SET` clause to determine the columns to
///     update with the provided values
pub struct DeleteQueryBuilder<'a, T, I>
where
    T: CrudOperations<T> + Transaction<T> + RowMapper<T>,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
    TransactionInput<'a>: From<&'a I>,
{
    _inner: QueryBuilder<'a, T, I>,
}

impl<'a, T, I> DeleteQueryBuilder<'a, T, I>
where
    T: CrudOperations<T> + Transaction<T> + RowMapper<T>,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
    TransactionInput<'a>: From<&'a I>,
{
    /// Generates a new public instance of the [`DeleteQueryBuilder`]
    pub fn new(table_schema_data: &str, input: I) -> Self {
        Self {
            _inner: QueryBuilder::<T, I>::new(
                Query::new(format!("DELETE FROM {table_schema_data}")),
                input,
            ),
        }
    }

    /// Launches the generated query to the database pointed by the
    /// selected datasource
    #[inline]
    pub async fn query(self) -> Result<Vec<T>, Box<(dyn std::error::Error + Sync + Send + 'a)>> {
        self._inner.query().await
    }
}

impl<'a, T, I> ops::QueryBuilder<'a, T> for DeleteQueryBuilder<'a, T, I>
where
    T: Debug + CrudOperations<T> + Transaction<T> + RowMapper<T> + Send,
    I: Into<TransactionInput<'a>> + Send + Sync + 'a,
    TransactionInput<'a>: From<&'a I>,
{
    #[inline]
    fn read_sql(&'a self) -> &'a str {
        self._inner.query.sql.as_str()
    }

    #[inline(always)]
    fn push_sql(mut self, sql: &str) {
        self._inner.query.sql.push_str(sql);
    }

    #[inline]
    fn r#where<Z: FieldValueIdentifier<'a, T>>(
        mut self,
        r#where: Z,
        op: impl Operator,
    ) -> Self {
        self._inner.r#where(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier<'a, T>>(mut self, column: Z, op: impl Operator) -> Self {
        self._inner.and(column, op);
        self
    }

    #[inline]
    fn and_values_in<Z, Q>(mut self, r#and: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier<T>,
        Q: QueryParameter<'a>,
    {
        self._inner.or_values_in(and, values);
        self
    }

    #[inline]
    fn or_values_in<Z, Q>(mut self, r#or: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier<T>,
        Q: QueryParameter<'a>,
    {
        self._inner.or_values_in(or, values);
        self
    }

    #[inline]
    fn or<Z: FieldValueIdentifier<'a, T>>(mut self, column: Z, op: impl Operator) -> Self {
        self._inner.or(column, op);
        self
    }

    #[inline]
    fn order_by<Z: FieldIdentifier<T>>(mut self, order_by: Z, desc: bool) -> Self {
        self._inner.order_by(order_by, desc);
        self
    }
}
