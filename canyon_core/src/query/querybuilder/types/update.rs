use crate::connection::database_type::DatabaseType;
use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::{Comp, Operator};
use crate::query::parameters::QueryParameter;
use crate::query::query::Query;
use crate::query::querybuilder::types::TableMetadata;
use crate::query::querybuilder::{QueryBuilder, QueryBuilderOps, QueryKind, UpdateQueryBuilderOps};
use std::error::Error;

/// Contains the specific database operations of the *UPDATE* SQL statements.
pub struct UpdateQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a>,
}

impl<'a> UpdateQueryBuilder<'a> {
    /// Generates a new public instance of the [`UpdateQueryBuilder`]
    pub fn new(
        table_schema_data: TableMetadata<'a>,
        database_type: DatabaseType,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>> {
        Ok(Self {
            _inner: QueryBuilder::new(table_schema_data, QueryKind::Update, database_type)?,
        })
    }

    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        self._inner.build()
    }
}

impl<'a> UpdateQueryBuilderOps<'a> for UpdateQueryBuilder<'a> {
    /// Creates an SQL `SET` clause to specify the columns that must be updated in the sentence
    fn set<Z, Q>(mut self, columns: &'a [(Z, Q)]) -> Self
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
    {
        if columns.is_empty() {
            // TODO: this is an err as well
            return self;
        }
        if self._inner.sql.contains("SET") {
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
                self._inner.params.len() + 1
            ));

            if idx < columns.len() - 1 {
                set_clause.push_str(", ");
            }
            self._inner.params.push(&column.1);
        }

        self._inner.sql.push_str(&set_clause);
        self
    }
}

impl<'a> QueryBuilderOps<'a> for UpdateQueryBuilder<'a> {
    #[inline]
    fn read_sql(&'a self) -> &'a str {
        self._inner.sql.as_str()
    }

    #[inline(always)]
    fn push_sql(mut self, sql: &str) {
        self._inner.sql.push_str(sql);
    }

    #[inline]
    fn r#where<Z: FieldValueIdentifier>(mut self, r#where: &'a Z, op: Comp) -> Self {
        self._inner.r#where(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: Comp) -> Self {
        self._inner.and(column, op);
        self
    }

    #[inline]
    fn and_values_in<Z, Q>(mut self, r#and: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Vec<&'a (dyn QueryParameter + 'a)>: Extend<&'a Q>,
    {
        self._inner.and_values_in(and, values);
        self
    }

    #[inline]
    fn or_values_in<Z, Q>(mut self, r#or: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Vec<&'a (dyn QueryParameter + 'a)>: Extend<&'a Q>,
    {
        self._inner.or_values_in(or, values);
        self
    }

    #[inline]
    fn or<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: Comp) -> Self {
        self._inner.or(column, op);
        self
    }

    #[inline]
    fn order_by<Z: FieldIdentifier>(mut self, order_by: Z, desc: bool) -> Self {
        self._inner.order_by(order_by, desc);
        self
    }
}
