//! Standalone functions that shares the same behaviour for different AST kinds

use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::clause::ConditionClause;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::symbol::Symbol;
use crate::query::querybuilder::syntax::symbol::Symbol::Comma;
use crate::query::querybuilder::syntax::tokens::{SqlTokens, ToSqlTokens};
use std::borrow::Cow;

pub(crate) struct Range(usize, Option<usize>);
impl Range {
    pub(crate) const fn new(start: usize, end: usize) -> Self {
        Self(start, Some(end))
    }

    /// Creates a new unbounded range starting from the given index.
    ///
    /// Here `None` does not mean infinity. It represents a single-value range:
    /// `[start, start]`.
    pub(crate) const fn new_unbounded(start: usize) -> Self {
        Self(start, None)
    }

    pub(crate) const fn is_range(&self) -> bool {
        self.1.is_some()
    }

    pub(crate) const fn start(&self) -> usize {
        self.0
    }

    pub(crate) const fn end(&self) -> usize {
        match self.1 {
            Some(end) => end,
            None => self.0,
        }
    }
}

impl<'a> IntoIterator for &'a Range {
    type Item = usize;
    type IntoIter = std::ops::Range<usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.start()..self.end()
    }
}

/// Helper function to push a quoted identifier (like table or column names) into the token stream
pub fn push_quoted_ident<'a, D, S>(element: S, tokens: &mut SqlTokens<'a>)
where
    D: SqlDialect,
    S: Into<Cow<'a, str>>,
{
    let q = D::IDENT_QUOTING;
    tokens.symbol(q.opening().into());
    tokens.ident(element);
    tokens.symbol(q.closing().into());
}

/// Helper function to emit a list of columns, separated by commas
pub(crate) fn emit_columns<'a, D: SqlDialect>(
    columns: &Vec<ColumnRef<'a>>,
    tokens: &mut SqlTokens<'a>,
) {
    if columns.is_empty() {
        tokens.symbol(Symbol::Asterisk);

        return;
    }

    for (i, column) in columns.iter().enumerate() {
        if i > 0 {
            tokens.symbol(Comma);
        }
        tokens.extend(<ColumnRef<'_> as ToSqlTokens<'_, D>>::to_tokens(column));
    }
}

pub(crate) fn emit_placeholders<'a>(columns: &Vec<ColumnRef<'a>>, tokens: &mut SqlTokens<'a>) {
    for (i, _) in columns.iter().enumerate() {
        if i > 0 {
            tokens.symbol(Comma);
        }
        tokens.placeholder();
    }
}

pub(crate) fn add_clause_conditions<'a, D: SqlDialect>(
    base_ast: &BaseAst<'a>,
    tokens: &mut SqlTokens<'a>,
) {
    if !base_ast.conditions.is_empty() {
        for cond in &base_ast.conditions {
            tokens.extend(<ConditionClause<'_> as ToSqlTokens<'_, D>>::to_tokens(cond));
        }
    }
}

pub(crate) fn emit_query_conditions<'a, D: SqlDialect>(
    query_conditions: &Vec<ConditionClause<'a>>,
    tokens: &mut SqlTokens<'a>,
) {
    if query_conditions.is_empty() {
        return;
    }

    for cond in query_conditions {
        tokens.extend(<ConditionClause<'_> as ToSqlTokens<'_, D>>::to_tokens(cond));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::querybuilder::syntax::ast::BaseAst;
    use crate::query::querybuilder::syntax::dialect::{
        IdentQuotingStyle, PlaceholderSymbol, StandardDialect,
    };
    use crate::query::querybuilder::syntax::table_metadata::TableMetadata;

    #[cfg(feature = "mssql")]
    use crate::query::querybuilder::syntax::dialect::MsSql;
    #[cfg(feature = "mysql")]
    use crate::query::querybuilder::syntax::dialect::MySql;
    #[cfg(feature = "postgres")]
    use crate::query::querybuilder::syntax::dialect::PgDialect;
    use crate::query::querybuilder::syntax::tokens::SqlToken;

    fn make_base_ast<'a>() -> BaseAst<'a> {
        BaseAst {
            table: TableMetadata::from("users"),
            conditions: vec![],
        }
    }

    fn make_column(column: &'_ str) -> ColumnRef<'_> {
        ColumnRef::from(column)
    }

    fn make_qualified_column<'a>(
        table: &'a str,
        column: &'a str,
        alias: Option<&'a str>,
    ) -> ColumnRef<'a> {
        ColumnRef {
            table: (!table.is_empty()).then_some(Cow::Borrowed(table)),
            column: Cow::Borrowed(column),
            alias: alias.map(Cow::Borrowed),
        }
    }

    fn assert_ident_quoting_contract<D: SqlDialect>(
        expected_opening: &str,
        expected_closing: &str,
    ) {
        assert_eq!(D::IDENT_QUOTING.opening().to_string(), expected_opening);
        assert_eq!(D::IDENT_QUOTING.closing().to_string(), expected_closing);
    }

    #[test]
    fn standard_dialect_uses_double_quotes_for_identifiers() {
        assert_eq!(
            StandardDialect::IDENT_QUOTING,
            IdentQuotingStyle::DoubleQuote
        );
        assert_ident_quoting_contract::<StandardDialect>("\"", "\"");
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn postgres_uses_double_quotes_for_identifiers() {
        assert_eq!(PgDialect::IDENT_QUOTING, IdentQuotingStyle::DoubleQuote);
        assert_ident_quoting_contract::<PgDialect>("\"", "\"");
    }

    #[cfg(feature = "mysql")]
    #[test]
    fn mysql_uses_backticks_for_identifiers() {
        assert_eq!(MySql::IDENT_QUOTING, IdentQuotingStyle::Backtick);
        assert_ident_quoting_contract::<MySql>("`", "`");
    }

    #[cfg(feature = "mssql")]
    #[test]
    fn mssql_uses_brackets_for_identifiers() {
        assert_eq!(MsSql::IDENT_QUOTING, IdentQuotingStyle::Bracket);
        assert_ident_quoting_contract::<MsSql>("[", "]");
    }

    #[test]
    fn push_quoted_ident_with_standard_dialect() {
        let mut tokens = SqlTokens::default();
        // TODO: this isn't taking in consideration the scape quotes, care
        push_quoted_ident::<StandardDialect, &str>("users", &mut tokens);
        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<StandardDialect>(&["users"])
        );
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn push_quoted_ident_with_postgres() {
        let mut tokens = SqlTokens::default();
        push_quoted_ident::<PgDialect, &str>("users", &mut tokens);
        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<PgDialect>(&["users"])
        );
    }

    fn get_columns_test_expr_values<D: SqlDialect>(
        literals: &[&'static str],
    ) -> Vec<SqlToken<'static>> {
        let mut tokens = SqlTokens::default();

        for (idx, lit) in literals.iter().enumerate() {
            if idx > 0 {
                tokens.symbol(Comma);
            }
            tokens.extend(<ColumnRef<'_> as ToSqlTokens<'_, D>>::to_tokens(
                &make_column(lit),
            ));
        }

        tokens.inner()
    }

    #[cfg(feature = "mysql")]
    #[test]
    fn push_quoted_ident_with_mysql() {
        let mut tokens = SqlTokens::default();
        push_quoted_ident::<MySql, &str>("users", &mut tokens);
        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<MySql>(&["users"])
        );
    }

    #[cfg(feature = "mssql")]
    #[test]
    fn push_quoted_ident_with_mssql() {
        let mut tokens = SqlTokens::default();
        push_quoted_ident::<MsSql, &str>("users", &mut tokens);
        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<MsSql>(&["users"])
        );
    }

    #[test]
    fn emit_columns_with_empty_vec_emits_asterisk() {
        let columns = vec![];
        let mut tokens = SqlTokens::default();
        emit_columns::<StandardDialect>(&columns, &mut tokens);
        assert_eq!(tokens.inner(), vec![SqlToken::Symbol(Symbol::Asterisk)]);
    }

    #[test]
    fn emit_columns_with_one_column_quotes_only_column_name_and_emit_column_alias() {
        let columns = vec![make_qualified_column("user", "name", Some("username"))];
        let mut tokens = SqlTokens::default();
        emit_columns::<StandardDialect>(&columns, &mut tokens);
        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<StandardDialect>(&["user.name as username"])
        );
    }

    #[test]
    fn emit_columns_with_many_columns_separates_with_comma_and_space() {
        let columns = vec![
            make_qualified_column("users", "id", None),
            make_qualified_column("users", "name", None),
            make_qualified_column("users", "email", None),
        ];
        let mut tokens = SqlTokens::default();
        emit_columns::<StandardDialect>(&columns, &mut tokens);

        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<StandardDialect>(&[
                "users.id",
                "users.name",
                "users.email"
            ])
        );
    }

    #[cfg(feature = "mysql")]
    #[test]
    fn emit_columns_with_mysql_uses_backticks() {
        let columns = vec![
            make_qualified_column("users", "id", None),
            make_qualified_column("users", "name", None),
        ];
        let mut tokens = SqlTokens::default();

        emit_columns::<MySql>(&columns, &mut tokens);

        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<MySql>(&["users.id", "users.name"])
        );
    }

    #[cfg(feature = "mssql")]
    #[test]
    fn emit_columns_with_mssql_uses_brackets() {
        let columns = vec![make_column("id"), make_column("name")];

        let mut tokens = SqlTokens::default();
        emit_columns::<MsSql>(&columns, &mut tokens);

        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<MsSql>(&["id", "name"])
        );
    }

    #[test]
    fn emit_columns_ignores_table_and_alias_and_only_emits_column_names() {
        let columns = vec![
            make_qualified_column("user", "id", None),
            make_qualified_column("account", "name", None),
        ];
        let mut tokens = SqlTokens::default();
        emit_columns::<StandardDialect>(&columns, &mut tokens);

        let expected =
            get_columns_test_expr_values::<StandardDialect>(&["user.id", "account.name"]);

        assert_eq!(tokens.inner(), expected);
    }

    fn get_placeholders_test_expr_values() -> Vec<SqlToken<'static>> {
        vec![
            SqlToken::Placeholder,
            SqlToken::Symbol(Comma),
            SqlToken::Placeholder,
        ]
    }

    #[cfg(feature = "mssql")]
    #[test]
    fn emit_placeholders_with_mssql_uses_at_p_numbering() {
        let columns = vec![make_column("id"), make_column("name"), make_column("email")];

        let mut tokens = SqlTokens::default();
        emit_placeholders(&columns, &mut tokens);
        let tokens_vec = tokens.inner();

        assert_eq!(
            &tokens_vec,
            &vec![
                SqlToken::Placeholder,
                SqlToken::Symbol(Comma),
                SqlToken::Placeholder,
                SqlToken::Symbol(Comma),
                SqlToken::Placeholder,
            ]
        );
        assert_eq!(
            tokens_vec
                .iter()
                .filter(|t| (*t).eq(&SqlToken::Placeholder))
                .count(),
            3
        );
        assert_eq!(MsSql::PLACEHOLDER_SYMBOL, PlaceholderSymbol::AtPNumbered);
    }
}
