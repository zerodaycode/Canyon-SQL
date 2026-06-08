pub mod delete;
pub mod select;
pub mod update;
mod insert;

pub use self::{delete::*, select::*, update::*};
use crate::{
    query::querybuilder::syntax::emitter::types::helpers::Range,
    connection::database_type::DatabaseType,
    query::{
        bounds::{FieldIdentifier, FieldValueIdentifier},
        operators::Operator,
        parameters::QueryParameter,
        query::Query,
        querybuilder::syntax::{
            ast::BaseAst, clause::ConditionClauseKind, emitter::AstProcessor,
            table_metadata::TableMetadata,
        },
    },
    query::ColumnRef
};
use std::error::Error;

/// Type for construct more complex queries than the classical CRUD ones.
pub struct QueryBuilder<'a, P: AstProcessor<'a> + 'a> {
    pub(crate) base_ast: BaseAst<'a>,
    pub(crate) ast: P,
    pub(crate) database_type: DatabaseType,
    pub(crate) params: Vec<&'a dyn QueryParameter>,
}

unsafe impl<'a, P: AstProcessor<'a>> Send for QueryBuilder<'a, P> {}
unsafe impl<'a, P: AstProcessor<'a>> Sync for QueryBuilder<'a, P> {}

impl<'a, P: AstProcessor<'a> + 'a> QueryBuilder<'a, P> {
    pub fn new(
        table_metadata: impl Into<TableMetadata<'a>>,
        ast: P,
        database_type: DatabaseType,
    ) -> Self {
        Self {
            base_ast: BaseAst::new(table_metadata),
            ast,
            database_type,
            params: Vec::new(),
        }
    }

    pub const fn new_querybuilder(
        table_metadata: TableMetadata<'a>,
        ast: P,
        database_type: DatabaseType,
    ) -> Self {
        Self {
            base_ast: BaseAst::new_ast(table_metadata),
            ast,
            database_type,
            params: Vec::new(),
        }
    }

    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        __impl::check_invariants_over_condition_clauses(&self)?;

        let Self {
            mut base_ast,
            ast,
            database_type,
            params,
        } = self;

        let sql = __detail::sql(database_type, &ast, &mut base_ast)?;
        Ok(Query::new(sql, params))
    }

    fn r#where<I: Into<ColumnRef<'a>>>(&mut self, column_name: I, operator: Operator) {
        __impl::create_condition_clause(self, ConditionClauseKind::Where, column_name, operator);
    }

    pub fn where_value<Z: FieldValueIdentifier>(&mut self, r#where: &'a Z, operator: Operator) {
        let (column_name, value) = r#where.value();
        self.params.push(value);
        __impl::create_condition_clause(self, ConditionClauseKind::Where, column_name, operator);
    }

    pub fn and<Z: FieldValueIdentifier>(&mut self, r#and: &'a Z, operator: Operator) {
        let (column_name, value) = r#and.value();
        self.params.push(value);
        __impl::create_condition_clause(self, ConditionClauseKind::And, column_name, operator);
    }

    pub fn and_values_in<'b, Z, Q>(
        &mut self,
        field: Z,
        values: &'a [Q],
    ) -> Result<(), Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
    {
        let actual_params_len = self.params.len();
        __impl::create_ranged_condition_clause(
            self,
            ConditionClauseKind::And,
            field.as_str(),
            Operator::In,
            Range::new(actual_params_len, actual_params_len + values.len()),
        );
        __impl::add_values_in_for_and_or_or_clause(self, ConditionClauseKind::And, field, values)
    }

    pub fn or_values_in<'b, Z, Q>(
        &mut self,
        r#or: Z,
        values: &'a [Q],
    ) -> Result<(), Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
    {
        let actual_params_len = self.params.len();
        __impl::create_ranged_condition_clause(
            self,
            ConditionClauseKind::Or,
            r#or.as_str(),
            Operator::In,
            Range::new(actual_params_len, actual_params_len + values.len()),
        );
        __impl::add_values_in_for_and_or_or_clause(self, ConditionClauseKind::Or, r#or, values)
    }

    pub fn or<Z: FieldValueIdentifier>(&mut self, r#or: &'a Z, operator: Operator) {
        let (column_name, value) = r#or.value();
        self.params.push(value);
        __impl::create_condition_clause(self, ConditionClauseKind::Or, column_name, operator);
    }
}

mod __impl {
    use crate::query::bounds::FieldIdentifier;
    use crate::query::operators::Operator;
    use crate::query::parameters::QueryParameter;
    use crate::query::querybuilder::QueryBuilder;
    use crate::query::querybuilder::syntax::clause::{ConditionClause, ConditionClauseKind};
    use crate::query::querybuilder::syntax::column::ColumnRef;
    use crate::query::querybuilder::syntax::emitter::AstProcessor;
    use crate::query::querybuilder::syntax::emitter::types::helpers::Range;
    use crate::query::querybuilder::types::__validators;
    use std::error::Error;

    pub(crate) fn add_values_in_for_and_or_or_clause<'a, 'b, P, Z, Q>(
        _self: &mut QueryBuilder<'a, P>,
        _conjunction_clause_kind: ConditionClauseKind,
        field: Z,
        values: &'a [Q],
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        Q: QueryParameter,
        Z: FieldIdentifier,
        P: AstProcessor<'a>,
    {
        let target_column = field.as_str();
        __validators::check_not_empty_in_clause_values(
            &_self.base_ast.table,
            target_column,
            values,
        )?;

        for value in values {
            _self.params.push(value);
        }

        Ok(())
    }

    /// Quick standalone that acts as a façade for an orchestrator that just organizes a procedural way of testing
    /// that the constructed underlying query is syntactically correct
    pub(crate) fn check_invariants_over_condition_clauses<'a, 'b, P: AstProcessor<'a>>(
        _self: &QueryBuilder<'a, P>,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'b>> {
        __validators::check_where_clause_position(_self)
    }

    pub(crate) fn create_condition_clause<'a, 'b, P: AstProcessor<'a>>(
        _self: &mut QueryBuilder<'a, P>,
        kind: ConditionClauseKind,
        column_name: impl Into<ColumnRef<'a>>,
        operator: Operator,
    ) {
        _self.base_ast.conditions.push(ConditionClause {
            kind,
            column_name: column_name.into(),
            operator,
            value_indexes: Some(Range::new_unbounded(_self.params.len())),
        });
    }

    pub(crate) fn create_ranged_condition_clause<'a, 'b, P: AstProcessor<'a>>(
        _self: &mut QueryBuilder<'a, P>,
        kind: ConditionClauseKind,
        column_name: impl Into<ColumnRef<'a>>,
        operator: Operator,
        value_indexes_range: Range,
    ) {
        _self.base_ast.conditions.push(ConditionClause {
            kind,
            column_name: column_name.into(),
            operator,
            value_indexes: Some(value_indexes_range),
        });
    }
}

mod __detail {
    use crate::connection::database_type::DatabaseType;
    use crate::query::querybuilder::syntax::ast::BaseAst;
    use crate::query::querybuilder::syntax::emitter::backends::PgEmitter;
    use crate::query::querybuilder::syntax::emitter::backends::{MySqlEmitter, SqlServerEmitter};
    use crate::query::querybuilder::syntax::emitter::{AstProcessor, SqlEmitter};
    use crate::query::querybuilder::syntax::tokens::SqlTokens;
    use crate::query::querybuilder::syntax::writer::TokenWriter;
    use std::error::Error;
    use std::fmt::Write;

    pub(super) fn sql<'a, P>(
        database_type: DatabaseType,
        ast: &P,
        base_ast: &mut BaseAst<'a>,
    ) -> Result<String, Box<dyn Error + Send + Sync + 'a>>
    where
        P: AstProcessor<'a>,
    {
        let tokens = run_emission_phase(database_type, ast, base_ast);
        run_render_phase(tokens, database_type)
    }

    /// Executes the SQL emission phase for the given AST and database backend.
    ///
    /// This function selects the appropriate backend-specific emitter
    /// based on `database_type` and delegates SQL generation to it.
    ///
    /// It acts as the orchestration boundary between:
    /// - The backend-agnostic query representation (`ast`, `base_ast`)
    /// - The backend-specific SQL emission strategy (`PgEmitter`, `MySqlEmitter`, etc.)
    ///
    /// # Parameters
    ///
    /// - `database_type`: Target database backend used to determine
    ///   which SQL dialect implementation will be executed.
    /// - `ast`: The query AST that describes the high-level structure
    ///   of the query.
    /// - `base_ast`: Shared base metadata required for emission,
    ///   such as table information and condition clauses.
    ///
    /// # Behavior
    ///
    /// The function:
    /// 1. Extracts the query kind from the AST.
    /// 2. Instantiates the corresponding backend emitter.
    /// 3. Executes the emission phase for that backend.
    ///
    /// # Panics
    ///
    /// Panics if the provided database backend is not supported.
    pub(super) fn run_emission_phase<'a, P>(
        database_type: DatabaseType,
        ast: &P,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a>
    where
        P: AstProcessor<'a>,
    {
        match database_type {
            DatabaseType::PostgreSql | DatabaseType::Deferred => {
                // TODO: review the semantics of this assumption
                let mut emitter = PgEmitter::default();
                emitter.emit(ast, base_ast)
            }
            DatabaseType::MySQL => {
                let mut emitter = MySqlEmitter::default();
                emitter.emit(ast, base_ast)
            }
            DatabaseType::SqlServer => {
                let mut emitter = SqlServerEmitter::default();
                emitter.emit(ast, base_ast)
            }
        }
    }

    /// Convenient standalone that helps us to interpolate the placeholder of the parameters of a SQL
    /// query directly into the passed in buffer, avoiding the need to construct and allocate temporary strings
    /// for such purpose
    pub(crate) fn _write_param_placeholder(
        db_type: DatabaseType,
        buffer: &mut String,
        params: impl Iterator,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(match db_type {
            DatabaseType::PostgreSql => write!(
                buffer,
                "${}",
                _calculate_param_placeholder_count_value(params)
            ),
            DatabaseType::SqlServer => write!(
                buffer,
                "@P{}",
                _calculate_param_placeholder_count_value(params)
            ),
            DatabaseType::MySQL => write!(buffer, "?"),
            _ => panic!("Provisional (placeholder)"),
        }?)
    }

    fn _calculate_param_placeholder_count_value(container: impl Iterator) -> usize {
        container.count()
    }

    pub(crate) fn run_render_phase<'a>(
        tokens: SqlTokens<'a>,
        db: DatabaseType,
    ) -> Result<String, Box<dyn Error + Send + Sync + 'a>> {
        let writer = TokenWriter::new();
        match db {
            DatabaseType::PostgreSql | DatabaseType::Deferred => writer.render::<PgEmitter>(tokens),
            DatabaseType::MySQL => writer.render::<MySqlEmitter>(tokens),
            DatabaseType::SqlServer => writer.render::<SqlServerEmitter>(tokens),
        }
        .map_err(|e| e.into())
    }
}

mod __validators {
    use crate::query::parameters::QueryParameter;
    use crate::query::querybuilder::QueryBuilder;
    use crate::query::querybuilder::syntax::clause::ConditionClauseKind;
    use crate::query::querybuilder::syntax::emitter::AstProcessor;
    use crate::query::querybuilder::types::__errors;
    use std::error::Error;
    use std::fmt::Display;

    /// For now, it's mandatory because we need to ensure what's the placeholder index which is the element
    /// that should swap with the where clause if isn't put in an incorrect order, no implementation ready
    pub(crate) fn check_where_clause_position<'a, 'b, P: AstProcessor<'a>>(
        _self: &QueryBuilder<'a, P>,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'b>> {
        if let Some(condition_clause) = &_self.base_ast.conditions.first()
            && condition_clause.kind.ne(&ConditionClauseKind::Where)
        {
            __errors::where_clause_position()
        } else {
            Ok(())
        }
    }

    pub(crate) fn check_not_empty_in_clause_values<'a, 'b, Q>(
        table_metadata: impl Display,
        column: &'a str,
        values: &'a [Q],
    ) -> Result<(), Box<dyn Error + Send + Sync + 'b>>
    where
        Q: QueryParameter,
    {
        if values.is_empty() {
            return __errors::empty_in_clause(table_metadata, column);
        }
        Ok(())
    }
}

mod __errors {
    use std::error::Error;
    use std::fmt::Display;
    use std::io::ErrorKind;

    pub(crate) fn where_clause_position<'a>() -> Result<(), Box<dyn Error + Send + Sync + 'a>> {
        Err(std::io::Error::new(
            // TODO: CanyonError
            ErrorKind::Unsupported,
            "Where clauses should be the first condition clause on a SQL sentence",
        )
        .into())
    }

    pub(crate) fn empty_in_clause<'a, 'b>(
        table_metadata: impl Display,
        column: &'a str,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'b>> {
        Err(std::io::Error::new( // TODO: CanyonError
            ErrorKind::Unsupported,
            format!("An IN clause has been added with empty values for {table_metadata} on the column: {column}", )).into())
    }
}
