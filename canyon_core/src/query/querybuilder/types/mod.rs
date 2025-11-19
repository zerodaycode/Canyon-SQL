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
use std::fmt::Write;
use crate::query::querybuilder::syntax::clause::{ConditionClause, ConditionClauseKind};
use crate::query::querybuilder::syntax::query_kind::QueryKind;
use crate::query::querybuilder::syntax::tokens::SqlToken;

/// Type for construct more complex queries than the classical CRUD ones.
pub struct QueryBuilder<'a> {
    pub(crate) meta: TableMetadata,
    pub(crate) kind: QueryKind,
    pub(crate) params: Vec<&'a dyn QueryParameter>,
    pub(crate) database_type: DatabaseType,
    pub(crate) condition_clauses: Vec<ConditionClause<'a>>,
}

unsafe impl Send for QueryBuilder<'_> {}
unsafe impl Sync for QueryBuilder<'_> {}

impl<'a> QueryBuilder<'a> {
    pub fn new(
        table_metadata: impl Into<TableMetadata>,
        kind: QueryKind,
        database_type: DatabaseType,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    {
        Ok(Self {
            meta: table_metadata.into(),
            kind,
            sql: String::new(),
            params: Vec::new(),
            database_type,
            condition_clauses: Vec::new(),
        })
    }

    /// Build for base QueryBuilder return Query using AST path according to kind.
    pub fn bui3ld(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        // validate invariants
        let qb = __impl::check_invariants_over_condition_clauses(self)?;

        // produce AST per kind
        let db = qb.database_type;
        return match qb.kind {
            QueryKind::Select => {
                // create SelectAst from qb minimal: columns/joins live in SelectQueryBuilder normally.
                // We produce an empty SelectAst from base and rely on SelectQueryBuilder::build to call SelectAst path.
                // To keep base behavior, we create a minimal SelectAst and let SelectQueryBuilder override.
                let mut ast = SelectAst::new(qb.meta.clone(), db);
                // copy base conditions & params
                ast.base.conditions = qb.condition_clauses;
                ast.base.params = qb.params;
                // emit tokens
                let mut tokens = Vec::<SqlToken>::new();
                ast.emit_tokens(&mut tokens);
                // now replace placeholders: for non-IN clauses we must convert each condition to placeholder indexes
                // Protocol: when AST emits operator tokens without placeholders, we append placeholders sequentially.
                // We'll implement helper to interleave placeholders
                let sql = __detail::render_tokens_with_placeholders(tokens, ast.base.database_type, &ast.base.params)?;
                Ok(Query::new(sql + ";", ast.base.params))
            }
            QueryKind::Update => {
                let mut ast = UpdateAst::new(qb.meta.clone(), db);
                ast.base.conditions = qb.condition_clauses;
                ast.base.params = qb.params;
                let mut tokens = Vec::<SqlToken>::new();
                ast.emit_tokens(&mut tokens);
                let sql = __detail::render_tokens_with_placeholders(tokens, ast.base.database_type, &ast.base.params)?;
                Ok(Query::new(sql + ";", ast.base.params))
            }
            QueryKind::Delete => {
                let ast = DeleteAst::new(qb.meta.clone(), db);
                // assign conditions and params
                let mut ast = ast;
                ast.base.conditions = qb.condition_clauses;
                ast.base.params = qb.params;
                let mut tokens = Vec::<SqlToken>::new();
                ast.emit_tokens(&mut tokens);
                let sql = __detail::render_tokens_with_placeholders(tokens, ast.base.database_type, &ast.base.params)?;
                Ok(Query::new(sql + ";", ast.base.params))
            }
            QueryKind::Insert => {
                let mut ast = InsertAst::new(qb.meta.clone(), db);
                ast.base.conditions = qb.condition_clauses;
                ast.base.params = qb.params;
                let mut tokens = Vec::<SqlToken>::new();
                ast.emit_tokens(&mut tokens);
                let sql = __detail::render_tokens_with_placeholders(tokens, ast.base.database_type, &ast.base.params)?;
                Ok(Query::new(sql + ";", ast.base.params))
            }
        }
    }

    /// Convenient SQL writer that starts all the appended SQL sentences by adding an initial empty
    /// whitespace
    pub fn push_sql(&mut self, part: &str) -> Result<(), Box<dyn Error + Send + Sync + 'a>> {
        write!(self.sql, " {}", part)?;
        Ok(())
    }

    /// Same as [Self::push_sql] but for adding char values to the underlying buffer
    pub fn push_sql_char(&mut self, part: char) -> Result<(), Box<dyn Error + Send + Sync + 'a>> {
        write!(self.sql, " {}", part)?;
        Ok(())
    }

    // pub fn build(mut self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
    //     self.sql.push_str(self.kind.as_ref());
    //
    //     let __self = __impl::check_invariants_over_condition_clauses(self)?;
    //     let mut __self = __impl::write_from_clause(__self)?;
    //
    //     __self.sql.push(';');
    //     Ok(Query::new(__self.sql, __self.params))
    // }

    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        let qb = __impl::check_invariants_over_condition_clauses(self)?;
        let mut tokens = Vec::<SqlToken>::new();

        __impl::emit_kind(&qb, &mut tokens);
        __impl::emit_from(&qb, &mut tokens);
        __impl::emit_conditions(&qb, &mut tokens);

        tokens.push(SqlToken::Symbol(';'));

        let sql = SqlToken::render_all(&tokens);
        Ok(Query::new(sql, qb.params))
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
        Vec<&'a (dyn QueryParameter + 'a)>: Extend<&'a Q>
    {
        __impl::generate_values_in_for_and_or_or_clause(self, ConditionClauseKind::And, field, values)?;
        Ok(())
    }

    pub fn or_values_in<'b, Z, Q>(&mut self, r#or: Z, values: &'a [Q])
        -> Result<(), Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Vec<&'a (dyn QueryParameter + 'a)>: Extend<&'a Q>
    {
        __impl::generate_values_in_for_and_or_or_clause(self, ConditionClauseKind::Or, r#or, values)?;
        Ok(())
    }

    pub fn or<Z: FieldValueIdentifier>(&mut self, r#or: &'a Z, operator: Comp) {
        let (column_name, value) = r#or.value();
        self.params.push(value);
        __impl::create_condition_clause(self, ConditionClauseKind::And, column_name, operator, value);
    }

    #[inline]
    pub fn order_by<Z: FieldIdentifier>(&mut self, order_by: Z, desc: bool) {
        self.sql.push_str(
            &(format!(
                " ORDER BY {}{}",
                order_by.as_str(),
                if desc { " DESC " } else { "" }
            )),
        );
    }

}


mod __impl {
    use std::borrow::Cow;
    use std::error::Error;
    use crate::query::querybuilder::types::__detail::write_param_placeholder;
    use crate::query::querybuilder::QueryBuilder;
    use std::fmt::Write;
    use crate::query::bounds::FieldIdentifier;
    use crate::query::operators::Comp;
    use crate::query::parameters::QueryParameter;
    use crate::query::querybuilder::syntax::clause::{ConditionClause, ConditionClauseKind};
    use crate::query::querybuilder::syntax::query_kind::QueryKind;
    use crate::query::querybuilder::syntax::tokens::{SqlToken, ToSqlTokens};
    use crate::query::querybuilder::types::__validators;

    pub(crate) fn emit_kind<'a>(qb: &'a QueryBuilder<'a>, out: &mut Vec<SqlToken<'a>>) {
        out.push(qb.kind.to_tokens());
    }

    pub(crate) fn emit_from<'a>(qb: &QueryBuilder<'a>, out: &mut Vec<SqlToken<'a>>) {
        match qb.kind {
            QueryKind::Select => {
                out.push(SqlToken::Symbol('*'));
                out.push(SqlToken::Keyword(Cow::from("FROM")));
                qb.meta.to_tokens(out);
            }
            QueryKind::Delete => {
                out.push(SqlToken::Keyword(std::borrow::Cow::Borrowed("FROM")));
                qb.meta.to_tokens(out);
            }
            QueryKind::Update => {
                qb.meta.to_tokens(out);
            }
        }
    }

    pub(crate) fn emit_conditions<'a>(
        qb: &QueryBuilder<'a>,
        out: &mut Vec<SqlToken<'a>>
    ) {
        for (i, c) in qb.condition_clauses.iter().enumerate() {
            c.to_tokens(out);
            out.push(SqlToken::Placeholder(i + 1));
        }
    }

    pub(crate) fn write_from_clause<'a>(mut _self: QueryBuilder<'a>) -> Result<QueryBuilder<'a>, Box<dyn Error + Send + Sync + 'a>> {
        if let Some(where_clause) = &_self.condition_clauses.first() {
            write!(_self.sql,
               " WHERE {} {}",
               where_clause.column_name,
               where_clause.operator
            )?;
            write_param_placeholder(_self.database_type, &mut _self.sql, _self.params.iter())?;
        }
        Ok(_self)
    }


    pub(crate) fn generate_values_in_for_and_or_or_clause<'a, Z, Q>(
        _self: &mut QueryBuilder<'a>,
        conjunction_clause_kind: ConditionClauseKind,
        field: Z,
        values: &'a [Q]
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        Q: QueryParameter,
        Z: FieldIdentifier,
        Vec<&'a dyn QueryParameter>: Extend<&'a Q>
    {
        let target_column = field.as_str();
        __validators::check_not_empty_in_clause_values(&_self.meta, target_column, values)?;

        _self.sql.push_str(" ");
        _self.sql.push_str(conjunction_clause_kind.as_ref());
        _self.sql.push_str(" ");
        _self.sql.push_str(target_column);
        _self.sql.push_str(" IN "); // TODO: was for reference, this is wrong
        _self.sql.push_str(ConditionClauseKind::In.as_ref());
        _self.sql.push_str(" (");

        let start = _self.params.len();
        let placeholders = (0..values.len())
            .map(|i| format!("${}", start + i + 1))
            .collect::<Vec<_>>()
            .join(", ");

        _self.sql.push_str(&placeholders);
        _self.sql.push(')');

        _self.params.extend(values);

        Ok(())
    }

    /// Quick standalone that acts as a façade for an orchestrator that just organizes a procedural way of testing
    /// that the constructed underlying query is syntactically correct
    pub(crate) fn check_invariants_over_condition_clauses<'a>(_self: QueryBuilder<'a>) -> Result<QueryBuilder<'a>, Box<dyn Error + Send + Sync + 'a>> {
        let _self = super::__validators::check_where_clause_position(_self)?;
        Ok(_self)
    }

    pub(crate) fn create_condition_clause<'a>(_self: &mut QueryBuilder<'a>, kind: ConditionClauseKind, column_name: &'a str, operator: Comp, value: &'a dyn QueryParameter) {
        _self.condition_clauses.push(
            ConditionClause {
                kind,
                column_name,
                operator,
                value
            });
    }
}


mod __detail {
    use crate::connection::database_type::DatabaseType;
    use std::error::Error;
    use std::fmt::Write;
    use crate::query::querybuilder::syntax::tokens::SqlToken;

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
    /// Render tokens into SQL string while inserting placeholders for param list.
    /// Strategy:
    ///  - tokens contain operators and identifiers; placeholders are inserted sequentially for simple operators (=, <, etc).
    ///  - For IN clauses that need multiple placeholders, the emitter previously emitted "(" and ")" and the caller
    ///    must ensure we place N placeholders where required. For simplicity here we map one placeholder per param.
    pub fn render_tokens_with_placeholders(tokens: Vec<SqlToken>, db: DatabaseType, params: &[&dyn crate::query::parameters::QueryParameter]) -> Result<String, Box<dyn Error + Send + Sync>> {
        // Simple approach: convert tokens to string and replace special marker tokens (Placeholder) if any.
        // Our tokens list uses SqlToken::Placeholder(usize) optionally; but many places emit operator and expect placeholders appended after.
        // For deterministic behavior, we'll transform tokens: when we encounter Operator and next token is not Placeholder, we will append placeholder using next param index.
        let mut out = String::with_capacity(256);
        let mut param_index = 0usize;

        let mut iter = tokens.into_iter().peekable();
        while let Some(tok) = iter.next() {
            match tok {
                SqlToken::Keyword(k) => { write!(out, " {}", k)?; }
                SqlToken::Ident(i) => { write!(out, " {}", i)?; }
                SqlToken::Symbol(s) => {
                    // symbols may be "," or "(" or ")"
                    // keep spacing rules: if s is "," then attach right after previous without leading space is acceptable
                    if s == "," || s == ")" {
                        write!(out, "{}", s)?;
                    } else {
                        write!(out, " {}", s)?;
                    }
                }
                // SqlToken::Operator(op) => {
                //     // write operator and then ensure a placeholder is generated unless next token is Placeholder or Symbol("(")
                //     write!(out, " {}", op)?;
                //     // peek next token
                //     if let Some(peek) = iter.peek() {
                //         match peek {
                //             SqlToken::Placeholder(_) => { /* will be rendered by next iteration */ }
                //             SqlToken::Symbol(sym) if *sym == "(" => {
                //                 // IN ( ) case: next parentheses contain placeholders managed by caller; skip auto placeholder.
                //             }
                //             _ => {
                //                 // inject placeholder for a single param
                //                 param_index += 1;
                //                 match db {
                //                     DatabaseType::PostgreSql => write!(out, " ${}", param_index)?,
                //                     DatabaseType::SqlServer => write!(out, " @P{}", param_index)?,
                //                     DatabaseType::MySQL => write!(out, " ?")?,
                //                     _ => write!(out, " ?")?,
                //                 }
                //             }
                //         }
                //     } else {
                //         // no next token => create placeholder
                //         param_index += 1;
                //         match db {
                //             DatabaseType::PostgreSql => write!(out, " ${}", param_index)?,
                //             DatabaseType::SqlServer => write!(out, " @P{}", param_index)?,
                //             DatabaseType::MySQL => write!(out, " ?")?,
                //             _ => write!(out, " ?")?,
                //         }
                //     }
                // }
                SqlToken::Placeholder(idx) => {
                    // direct placeholder -- render according to DB, using provided idx
                    match db {
                        DatabaseType::PostgreSql => write!(out, " ${}", idx)?,
                        DatabaseType::SqlServer => write!(out, " @P{}", idx)?,
                        DatabaseType::MySQL => write!(out, " ?")?,
                        _ => write!(out, " ?")?,
                    }
                }
                // SqlToken::Number(n) => { write!(out, " {}", n)?; }
                // SqlToken::Raw(s) => { write!(out, " {}", s)?; }
            }
        }

        // trim leading space
        let result = if out.starts_with(' ') { out[1..].to_string() } else { out };
        Ok(result)
    }
}

mod __validators {
    use std::error::Error;
    use std::fmt::Display;
    use crate::query::parameters::QueryParameter;
    use crate::query::querybuilder::QueryBuilder;
    use crate::query::querybuilder::syntax::clause::ConditionClauseKind;
    use crate::query::querybuilder::types::__errors;

    pub(crate) fn check_where_clause_position<'a>(_self: QueryBuilder<'a>) -> Result< QueryBuilder<'a>, Box<dyn Error + Send + Sync + 'a>> {
        if let Some(condition_clause) = &_self.condition_clauses.first() {
            if condition_clause.kind.ne(&ConditionClauseKind::Where) { // TODO: decide if we just re-organize the condition clauses
                return __errors::where_clause_position()
            }
        }
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
    

    pub(crate) fn where_clause_position<'a>() -> Result<QueryBuilder<'a>, Box<dyn Error + Send + Sync + 'a>> {
        return Err(std::io::Error::new( // TODO: CanyonError
            ErrorKind::Unsupported,
            "Where clauses should be the first condition clause on a SQL sentence").into())
    }

    pub(crate) fn empty_in_clause<'a, 'b>(table_metadata: impl Display, column: &'a str) -> Result<(), Box<dyn Error + Send + Sync + 'b>> {
        return Err(std::io::Error::new( // TODO: CanyonError
            ErrorKind::Unsupported,
            format!("An IN clause has been added with empty values for {table_metadata} on the column: {column}", )).into())
    }
}
