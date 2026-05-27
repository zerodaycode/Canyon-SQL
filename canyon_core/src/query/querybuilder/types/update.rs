use crate::connection::database_type::DatabaseType;
use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Operator;
use crate::query::parameters::QueryParameter;
use crate::query::query::Query;
use crate::query::querybuilder::syntax::ast::update::UpdateAst;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::types::TableMetadata;
use crate::query::querybuilder::{QueryBuilder, QueryBuilderOps, UpdateQueryBuilderOps};
use std::error::Error;

/// Contains the specific database operations of the *UPDATE* SQL statements.
pub struct UpdateQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a, UpdateAst<'a>>,
}

impl<'a> UpdateQueryBuilder<'a> {
    /// Generates a new public instance of the [`UpdateQueryBuilder`]
    pub fn new(
        table_schema_data: impl Into<TableMetadata<'a>>,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>> {
        UpdateQueryBuilder::new_for(table_schema_data, DatabaseType::Deferred)
    }

    pub fn new_for(
        table_schema_data: impl Into<TableMetadata<'a>>,
        database_type: DatabaseType,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>> {
        Ok(Self {
            _inner: QueryBuilder::new(table_schema_data, UpdateAst::new(), database_type)?,
        })
    }

    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        self._inner.build()
    }
}

impl<'a> UpdateQueryBuilderOps<'a> for UpdateQueryBuilder<'a> {
    fn set<I: Into<ColumnRef<'a>>>(
        mut self,
        columns: Vec<I>,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    where
        Self: std::marker::Sized,
    {
        __validators::set_clause_values_not_empty(&columns)?;
        self._inner.ast.columns = columns.into_iter().map(|f| f.into()).collect::<Vec<_>>();
        Ok(self)
    }

    fn set_values<Z, Q>(
        mut self,
        columns: &'a [(Z, Q)],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    where
        Z: FieldIdentifier + Into<ColumnRef<'a>> + Clone,
        Q: QueryParameter,
    {
        __validators::set_clause_not_already_present(&self)?;
        __validators::set_clause_values_not_empty(columns)?;

        // normalized column names
        self._inner.ast.columns = columns.iter().map(|(z, _)| <Z as Into<ColumnRef>>::into(z.clone())).collect::<Vec<_>>();

        // normalized values
        for (_, v) in columns {
            self._inner.params.push(v as &dyn QueryParameter);
        }

        Ok(self)
    }
}

impl<'a> QueryBuilderOps<'a> for UpdateQueryBuilder<'a> {
    #[inline]
    fn r#where(mut self, column_name: &'a str, operator: Operator) -> Self {
        self._inner.r#where(column_name, operator);
        self
    }

    #[inline]
    fn where_value<Z: FieldValueIdentifier>(mut self, r#where: &'a Z, op: Operator) -> Self {
        self._inner.where_value(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: Operator) -> Self {
        self._inner.and(column, op);
        self
    }

    #[inline]
    fn and_values_in<'b, Z, Q>(
        mut self,
        r#and: Z,
        values: &'a [Q],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
    {
        self._inner.and_values_in(and, values)?;
        Ok(self)
    }

    #[inline]
    fn or_values_in<'b, Z, Q>(
        mut self,
        r#or: Z,
        values: &'a [Q],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
    {
        self._inner.or_values_in(or, values)?;
        Ok(self)
    }

    #[inline]
    fn or<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: Operator) -> Self {
        self._inner.or(column, op);
        self
    }
}

mod __validators {
    use crate::query::querybuilder::UpdateQueryBuilder;
    use std::error::Error;
    use std::io::ErrorKind;

    pub(super) fn set_clause_not_already_present<'a>(
        _self: &UpdateQueryBuilder<'a>,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'a>> {
        if !_self._inner.ast.columns.is_empty() {
            return Err(std::io::Error::new(
                // TODO: CanyonError
                ErrorKind::Unsupported,
                "SET clause already present",
            )
            .into());
        }
        Ok(())
    }

    pub(super) fn set_clause_values_not_empty<T>(
        values: &[T],
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if values.is_empty() {
            return Err(std::io::Error::new(
                // TODO: CanyonError
                ErrorKind::Unsupported,
                "Empty SET clause",
            )
            .into());
        }
        Ok(())
    }
}
