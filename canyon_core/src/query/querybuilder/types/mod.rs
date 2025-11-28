pub mod delete;
pub mod select;
pub mod update;

use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
pub use self::{delete::*, select::*, update::*};
use crate::connection::database_type::DatabaseType;
use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Comp;
use crate::query::parameters::QueryParameter;
use crate::query::query::Query;
use std::error::Error;
use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::clause::ConditionClauseKind;
use crate::query::querybuilder::syntax::emitter::{AstProcessor, ToSql};
use crate::query::querybuilder::syntax::tokens::{SqlToken, Symbol, TokenWriter};

/// Type for construct more complex queries than the classical CRUD ones.
pub struct QueryBuilder<'a, P: AstProcessor + ToSql<'a>> {
    pub(crate) base_ast: BaseAst<'a>,
    pub(crate) ast: P,
    pub(crate) database_type: DatabaseType,
    pub(crate) params: Vec<&'a dyn QueryParameter>,
}

unsafe impl<'a, P: AstProcessor + ToSql<'a>> Send for QueryBuilder<'a, P> {}
unsafe impl<'a, P: AstProcessor + ToSql<'a>> Sync for QueryBuilder<'a, P> {}

impl<'a, P: AstProcessor + ToSql<'a>> QueryBuilder<'a, P> {
    pub fn new(
        table_metadata: impl Into<TableMetadata<'a>>,
        ast: P,
        database_type: DatabaseType,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    {
        Ok(Self {
            base_ast: BaseAst::new(table_metadata),
            ast,
            database_type,
            params: Vec::new(),
        })
    }

    pub fn build<'b: 'a>(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'b>> {
        // let qb = __impl::check_invariants_over_condition_clauses(self)?;

        let mut tokens = Vec::<SqlToken>::new();
        //let emitter = QueryEmitter::new_with_ast(QueryKind::Select, self.ast);
        // TODO: solve the diamond with the base_at and the emitter parameters
        // let emitter = QueryEmitter::Select(self.ast);emitter.emit_all_phases(&self.base_ast.table, &self.base_ast.conditions, &mut tokens);
        // emitter.emit_kind(&mut tokens);
        // emitter.emit_body(&mut tokens);
        self.ast.emit_all(&self.base_ast.table, &mut tokens);
        tokens.push(SqlToken::Symbol(Symbol::Semicolon)); // end with semicolon

        println!("QB TOKENS!: {:?}", tokens);

        let sql = TokenWriter::new().render(&tokens, &self.database_type)?;
        println!("QB str!: {:?}", sql);
        Ok(Query::new(sql, self.params)) // TODO, get rid out of query?
    }

    fn r#where(&mut self, column_name: &'a str, operator: Comp, value: &'a dyn QueryParameter) {
        __impl::create_condition_clause(self, ConditionClauseKind::Where, column_name, operator, value);
    }

    pub fn where_value<Z: FieldValueIdentifier>(&mut self, r#where: &'a Z, operator: Comp) {
        let (column_name, value) = r#where.value();
        self.params.push(value);
        __impl::create_condition_clause(self, ConditionClauseKind::Where, column_name, operator, value);
    }

    pub fn and<Z: FieldValueIdentifier>(&mut self, r#and: &'a Z, operator: Comp) {
        let (column_name, value) = r#and.value();
        self.params.push(value);
        __impl::create_condition_clause(self, ConditionClauseKind::And, column_name, operator, value);
    }

    pub fn and_values_in<'b, Z, Q>(&mut self, field: Z, values: &'a [Q])
    -> Result<(), Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
    {
        __impl::generate_values_in_for_and_or_or_clause(self, ConditionClauseKind::And, field, values)
    }

    pub fn or_values_in<'b, Z, Q>(&mut self, r#or: Z, values: &'a [Q])
        -> Result<(), Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
    {
        __impl::generate_values_in_for_and_or_or_clause(self, ConditionClauseKind::Or, r#or, values)
    }

    pub fn or<Z: FieldValueIdentifier>(&mut self, r#or: &'a Z, operator: Comp) {
        let (column_name, value) = r#or.value();
        self.params.push(value);
        __impl::create_condition_clause(self, ConditionClauseKind::And, column_name, operator, value);
    }

    #[inline]
    pub fn order_by<Z: FieldIdentifier>(&mut self, order_by: Z, desc: bool) {
        // self.sql.push_str(
        //     &(format!(
        //         " ORDER BY {}{}",
        //         order_by.as_str(),
        //         if desc { " DESC " } else { "" }
        //     )),
        // );
    }

}


mod __impl {
    use std::error::Error;
    use crate::query::querybuilder::QueryBuilder;
    use crate::query::bounds::FieldIdentifier;
    use crate::query::operators::Comp;
    use crate::query::parameters::QueryParameter;
    use crate::query::querybuilder::syntax::clause::{ConditionClause, ConditionClauseKind};
    use crate::query::querybuilder::syntax::emitter::{AstProcessor, ToSql};
    use crate::query::querybuilder::syntax::tokens::SqlToken;
    // pub(crate) fn write_from_clause<'a>(mut _self: QueryBuilder<P>) -> Result<QueryBuilder<P>, Box<dyn Error + Send + Sync + 'a>> {
    //     if let Some(where_clause) = &_self.condition_clauses.first() {
    //         write!(_self.sql,
    //            " WHERE {} {}",
    //            where_clause.column_name,
    //            where_clause.operator
    //         )?;
    //         write_param_placeholder(_self.database_type, &mut _self.sql, _self.params.iter())?;
    //     }
    //     Ok(_self)
    // }

    pub(crate) fn generate_values_in_for_and_or_or_clause<'a, 'b, P, Z, Q>(
        _self: &mut QueryBuilder<'a, P>,
        conjunction_clause_kind: ConditionClauseKind,
        field: Z,
        values: &'a [Q]
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        Q: QueryParameter,
        Z: FieldIdentifier, P: AstProcessor + ToSql<'a>
        // Vec<&'a dyn QueryParameter>: Extend<&'a Q>
    {
        // let target_column = field.as_str();
        // __validators::check_not_empty_in_clause_values(&_self.meta, target_column, values)?;
        //
        // _self.sql.push(' ');
        // _self.sql.push_str(conjunction_clause_kind.as_ref());
        // _self.sql.push(' ');
        // _self.sql.push_str(target_column);
        // _self.sql.push_str(" IN "); // TODO: was for reference, this is wrong
        // _self.sql.push_str(ConditionClauseKind::In.as_ref());
        // _self.sql.push_str(" (");
        //
        // let start = _self.params.len();
        // let placeholders = (0..values.len())
        //     .map(|i| format!("${}", start + i + 1))
        //     .collect::<Vec<_>>()
        //     .join(", ");
        //
        // _self.sql.push_str(&placeholders);
        // _self.sql.push(')');
        //
        // for value in values {
        //     _self.params.push(value);
        // }

        Ok(())
    }

    /// Quick standalone that acts as a façade for an orchestrator that just organizes a procedural way of testing
    /// that the constructed underlying query is syntactically correct
    pub(crate) fn check_invariants_over_condition_clauses<'a, 'b: 'a, P: AstProcessor + ToSql<'a>>(_self: QueryBuilder<'a, P>) -> Result<QueryBuilder<'a, P>, Box<dyn Error + Send + Sync + 'a>> {
        let _self = super::__validators::check_where_clause_position(_self)?;
        Ok(_self)
    }

    pub(crate) fn create_condition_clause<'a, 'b, P: AstProcessor + ToSql<'a>>(_self: &mut QueryBuilder<'a, P>, kind: ConditionClauseKind, column_name: &'a str, operator: Comp, value: &'a dyn QueryParameter) {
        _self.base_ast.conditions.push(
            ConditionClause {
                kind,
                column_name: SqlToken::new_ident(column_name),
                operator,
                value
            });
    }
}


mod __detail {
    use crate::connection::database_type::DatabaseType;
    use std::error::Error;
    use std::fmt::Write;

    /// Convenient standalone that helps us to interpolate the placeholder of the parameters of a SQL
    /// query directly into the passed in buffer, avoiding the need to construct and allocate temporary strings
    /// for such purpose
    pub(crate) fn write_param_placeholder(db_type: DatabaseType, buffer: &mut String, params: impl Iterator) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(match db_type{
            DatabaseType::PostgreSql => write!(buffer, "${}", calculate_param_placeholder_count_value(params)),
            DatabaseType::SqlServer => write!(buffer, "@P{}", calculate_param_placeholder_count_value(params)),
            DatabaseType::MySQL => write!(buffer, "?"),
            _ => panic!("Provisional (placeholder)"),
        }?)
    }

    fn calculate_param_placeholder_count_value(container: impl Iterator) -> usize{
        container.count()
    }
}

mod __validators {
    use std::error::Error;
    use std::fmt::Display;
    use crate::query::parameters::QueryParameter;
    use crate::query::querybuilder::QueryBuilder;
    use crate::query::querybuilder::syntax::emitter::{AstProcessor, ToSql};
    use crate::query::querybuilder::types::__errors;

    pub(crate) fn check_where_clause_position<'a, 'b, P: AstProcessor + ToSql<'a>>(_self: QueryBuilder<'a, P>) -> Result< QueryBuilder<'a, P>, Box<dyn Error + Send + Sync + 'a>> {
        // if let Some(condition_clause) = &_self.ast.condition_clauses.first() && condition_clause.kind.ne(&ConditionClauseKind::Where) {
        //         __errors::where_clause_position()
        // } else {
        // Ok(_self)}
        Ok(_self)
    }

    pub(crate) fn check_not_empty_in_clause_values<'a, 'b, Q>(table_metadata: impl Display, column: &'a str, values: &'a [Q])
        -> Result<(), Box<dyn Error + Send + Sync + 'b>>
    where
        Q: QueryParameter {
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
    
    use crate::query::querybuilder::QueryBuilder;
    use crate::query::querybuilder::syntax::emitter::{AstProcessor, ToSql};

    pub(crate) fn where_clause_position<'a, 'b, P: AstProcessor + ToSql<'a>>() -> Result<QueryBuilder<'a, P>, Box<dyn Error + Send + Sync + 'a>> {
        Err(std::io::Error::new( // TODO: CanyonError
            ErrorKind::Unsupported,
            "Where clauses should be the first condition clause on a SQL sentence").into())
    }

    pub(crate) fn empty_in_clause<'a, 'b>(table_metadata: impl Display, column: &'a str) -> Result<(), Box<dyn Error + Send + Sync + 'b>> {
        Err(std::io::Error::new( // TODO: CanyonError
            ErrorKind::Unsupported,
            format!("An IN clause has been added with empty values for {table_metadata} on the column: {column}", )).into())
    }
}
