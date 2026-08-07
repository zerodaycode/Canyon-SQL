use crate::{
    connection::database_type::DatabaseType,
    query::{
        bounds::{FieldIdentifier, FieldValueIdentifier},
        operators::Operator,
        parameters::QueryParameter,
        query::Query,
        querybuilder::{
            QueryBuilder, QueryBuilderOps, UpdateQueryBuilderOps,
            syntax::{ast::update::UpdateAst, column::ColumnRef},
            types::TableMetadata,
        },
    },
};
use std::error::Error;

/// Fluent builder for `UPDATE` statements
pub struct UpdateQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a, UpdateAst<'a>>,
}
impl<'a> UpdateQueryBuilder<'a> {
    /// Creates an update builder whose database dialect will be resolved later.
    pub fn new(
        table_schema_data: impl Into<TableMetadata<'a>>,
        database_type: DatabaseType,
    ) -> Self {
        Self {
            _inner: QueryBuilder::new(table_schema_data, UpdateAst::new(), database_type),
        }
    }
    /// Creates an update builder for a specific database dialect.
    pub fn new_for(table_schema_data: TableMetadata<'a>, database_type: DatabaseType) -> Self {
        Self {
            _inner: QueryBuilder::new_querybuilder(
                table_schema_data,
                UpdateAst::new(),
                database_type,
            ),
        }
    }
}

impl<'a> UpdateQueryBuilderOps<'a> for UpdateQueryBuilder<'a> {
    fn set<I: Into<ColumnRef<'a>>>(
        mut self,
        columns: Vec<I>,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    where
        Self: Sized,
    {
        __validators::set_clause_values_not_empty(&columns)?;
        self._inner.ast.columns = columns.into_iter().map(Into::into).collect();
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
        self._inner.ast.columns = columns
            .iter()
            .map(|(column, _)| column.clone().into())
            .collect();
        for (_, value) in columns {
            self._inner.params.push(value as &dyn QueryParameter);
        }
        Ok(self)
    }
}

impl<'a> QueryBuilderOps<'a> for UpdateQueryBuilder<'a> {
    #[inline(always)]
    fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        self._inner.build()
    }
    #[inline]
    fn r#where<I: Into<ColumnRef<'a>>>(mut self, column_name: I, operator: Operator) -> Self {
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
        self._inner.and_values_in(r#and, values)?;
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
        self._inner.or_values_in(r#or, values)?;
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

    /// Prevents `set_values` from replacing a previously configured `SET` clause.
    pub(super) fn set_clause_not_already_present<'a>(
        builder: &UpdateQueryBuilder<'a>,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'a>> {
        if !builder._inner.ast.columns.is_empty() {
            return Err(std::io::Error::new(
                // TODO: CanyonError
                ErrorKind::Unsupported,
                "SET clause already present",
            )
            .into());
        }
        Ok(())
    }

    /// Rejects update statements that would produce an empty `SET` clause.
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
