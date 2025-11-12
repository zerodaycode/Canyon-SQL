use crate::connection::database_type::DatabaseType;
use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Comp;
use crate::query::parameters::QueryParameter;
use crate::query::query::Query;
use crate::query::querybuilder::types::TableMetadata;
use crate::query::querybuilder::{QueryBuilder, QueryBuilderOps, QueryKind, UpdateQueryBuilderOps};
use std::error::Error;
use crate::canyon::Canyon;

/// Contains the specific database operations of the *UPDATE* SQL statements.
pub struct UpdateQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a>,
    pub(crate) columns: &'a [String],
}

impl<'a> UpdateQueryBuilder<'a> {
    /// Generates a new public instance of the [`UpdateQueryBuilder`]
    pub fn new(
        table_schema_data: &'a TableMetadata,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>> {
        Self::new_for(table_schema_data, Canyon::instance()?.get_default_db_type()?)
    }

    pub fn new_for(
        table_schema_data: &'a TableMetadata,
        database_type: DatabaseType,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>> {
        Ok(Self {
            _inner: QueryBuilder::new(table_schema_data, QueryKind::Update, database_type)?,
            columns: &[]
        })
    }

    pub fn build(mut self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        __impl::create_set_clause_columns_with_placeholders(&mut self);
        self._inner.build()
    }
}

impl<'a> UpdateQueryBuilderOps<'a> for UpdateQueryBuilder<'a> {
    fn set(mut self, columns: &'a [String]) -> Result<Self, Box<dyn Error + Send + Sync + 'a>> where Self: std::marker::Sized {
        __validators::set_clause_values_not_empty(columns)?;
        self.columns = columns;
        Ok(self)
    }

    fn set_with_values<Z, Q>(mut self, columns: &'a [(Z, Q)]) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Vec<&'a dyn QueryParameter>: Extend<&'a Q>
    {
        __validators::set_clause_not_already_present(&self)?;
        __validators::set_clause_values_not_empty(columns)?;

        self._inner.params.extend(columns.iter().map(|(_l, value)| value));

        Ok(self)
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
    fn r#where(mut self, column_name: &'a str, operator: Comp, value: &'a dyn QueryParameter) -> Self {
        self._inner.r#where(column_name, operator, value);
        self
    }

    #[inline]
    fn where_value<Z: FieldValueIdentifier>(mut self, r#where: &'a Z, op: Comp) -> Self {
        self._inner.where_value(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: Comp) -> Self {
        self._inner.and(column, op);
        self
    }

    #[inline]
    fn and_values_in<'b, Z, Q>(mut self, r#and: Z, values: &'a [Q]) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Vec<&'a (dyn QueryParameter + 'a)>: Extend<&'a Q>,
    {
        self._inner.and_values_in(and, values)?;
        Ok(self)
    }

    #[inline]
    fn or_values_in<'b, Z, Q>(mut self, r#or: Z, values: &'a [Q]) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Vec<&'a (dyn QueryParameter + 'a)>: Extend<&'a Q>,
    {
        self._inner.or_values_in(or, values)?;
        Ok(self)
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

mod __impl {
    use crate::query::querybuilder::UpdateQueryBuilder;

    pub(super) fn create_set_clause_columns_with_placeholders(_self: &mut UpdateQueryBuilder) {
        let mut set_clause = String::new();
        set_clause.push_str(" SET ");

        for (idx, column) in _self.columns.iter().enumerate() {
            set_clause.push_str(&format!(
                "{} = ${}",
                column,
                _self._inner.params.len() + 1
            ));

            if idx < _self.columns.len() - 1 {
                set_clause.push_str(", ");
            }
        }
    }
}

mod __validators {
    use std::error::Error;
    use std::io::ErrorKind;
    use crate::query::querybuilder::UpdateQueryBuilder;

    pub(super) fn set_clause_not_already_present<'a>(_self: &UpdateQueryBuilder<'a>) -> Result<(), Box<dyn Error + Send + Sync + 'a>> {
        if !_self.columns.is_empty() {
            return Err(std::io::Error::new( // TODO: CanyonError
                    ErrorKind::Unsupported,
                    "SET clause already present").into())
        }
        Ok(())
    }

    pub(super) fn set_clause_values_not_empty<T>(values: &[T]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if values.is_empty() {
            return Err(std::io::Error::new( // TODO: CanyonError
                ErrorKind::Unsupported,
                "Empty SET clause").into())
        }
        Ok(())
    }
}