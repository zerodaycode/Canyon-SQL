//! Standalone functions that shares the same behaviour for different AST kinds

use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::symbol::Symbol;
use crate::query::querybuilder::syntax::symbol::Symbol::Comma;
use crate::query::querybuilder::syntax::tokens::{PlaceholderKind, SqlTokens};

/// Helper function to push a quoted identifier (like table or column names) into the token stream
pub fn push_quoted_ident<'a, D: SqlDialect>(element: &'a str, tokens: &mut SqlTokens<'a>) {
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
        tokens.whitespace();
        return;
    }

    for (i, column) in columns.iter().enumerate() {
        if i > 0 {
            tokens.symbol(Comma);
            tokens.whitespace();
        }
        push_quoted_ident::<D>(column.column, tokens);
    }
}

pub(crate) fn emit_placeholders<'a>(
    columns: &Vec<ColumnRef<'a>>,
    base_ast: &mut BaseAst<'a>,
    tokens: &mut SqlTokens<'a>,
) {
    for (i, _) in columns.iter().enumerate() {
        if i > 0 {
            tokens.symbol(Comma);
            tokens.whitespace();
        }
        tokens.placeholder(PlaceholderKind::Value(base_ast.next_placeholder_index()));
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
    use crate::query::querybuilder::syntax::tokens::{PlaceholderKind, SqlToken};

    fn make_base_ast<'a>() -> BaseAst<'a> {
        BaseAst {
            table: TableMetadata::from("users"),
            conditions: vec![],
            bind_index: 0,
        }
    }

    fn make_column(column: &'_ str) -> ColumnRef<'_> {
        ColumnRef {
            table: None,
            column,
            alias: None,
        }
    }

    fn make_qualified_column<'a>(
        table: &'a str,
        column: &'a str,
        alias: Option<&'a str>,
    ) -> ColumnRef<'a> {
        ColumnRef {
            table: Some(table),
            column,
            alias,
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
        push_quoted_ident::<StandardDialect>("users", &mut tokens);
        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<StandardDialect>(&["users"])
        );
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn push_quoted_ident_with_postgres() {
        let mut tokens = SqlTokens::default();
        push_quoted_ident::<PgDialect>("users", &mut tokens);
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
            push_quoted_ident::<D>(lit, &mut tokens);
            if idx + 1 < literals.len() {
                tokens.extend([
                    SqlToken::Symbol(Comma),
                    SqlToken::WhiteSpace,
                ]);
            }
        }

        tokens.inner()
    }

    #[cfg(feature = "mysql")]
    #[test]
    fn push_quoted_ident_with_mysql() {
        let mut tokens = SqlTokens::default();
        push_quoted_ident::<MySql>("users", &mut tokens);
        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<MySql>(&["users"])
        );
    }

    #[cfg(feature = "mssql")]
    #[test]
    fn push_quoted_ident_with_mssql() {
        let mut tokens = SqlTokens::default();
        push_quoted_ident::<MsSql>("users", &mut tokens);
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
        assert_eq!(
            tokens.inner(),
            vec![SqlToken::Symbol(Symbol::Asterisk), SqlToken::WhiteSpace]
        );
    }

    #[test]
    fn emit_columns_with_one_column_quotes_only_column_name() {
        let columns = vec![make_qualified_column("users", "name", Some("username"))];
        let mut tokens = SqlTokens::default();
        emit_columns::<StandardDialect>(&columns, &mut tokens);
        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<StandardDialect>(&["name"])
        );
    }

    #[test]
    fn emit_columns_with_many_columns_separates_with_comma_and_space() {
        let columns = vec![make_column("id"), make_column("name"), make_column("email")];
        let mut tokens = SqlTokens::default();

        emit_columns::<StandardDialect>(&columns, &mut tokens);

        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<StandardDialect>(&["id", "name", "email"])
        );
    }

    #[cfg(feature = "mysql")]
    #[test]
    fn emit_columns_with_mysql_uses_backticks() {
        let columns = vec![make_column("id"), make_column("name")];
        let mut tokens = SqlTokens::default();

        emit_columns::<MySql>(&columns, &mut tokens);

        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<MySql>(&["id", "name"])
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
            make_qualified_column("users", "id", Some("user_id")),
            make_qualified_column("accounts", "name", Some("account_name")),
        ];
        let mut tokens = SqlTokens::default();

        emit_columns::<StandardDialect>(&columns, &mut tokens);

        assert_eq!(
            tokens.inner(),
            get_columns_test_expr_values::<StandardDialect>(&["id", "name"])
        );
    }

    #[test]
    fn emit_placeholders_with_empty_vec_emits_nothing_and_keeps_bind_index() {
        let columns = vec![];
        let mut base_ast = make_base_ast();
        let mut tokens = SqlTokens::default();

        emit_placeholders(&columns, &mut base_ast, &mut tokens);

        assert!(tokens.is_empty());
        assert_eq!(base_ast.bind_index, 0);
    }

    #[test]
    fn emit_placeholders_with_single_column_emits_first_placeholder() {
        let columns = vec![make_column("name")];
        let mut base_ast = make_base_ast();
        let mut tokens = SqlTokens::default();

        emit_placeholders(&columns, &mut base_ast, &mut tokens);

        assert_eq!(
            tokens.inner(),
            vec![SqlToken::Placeholder(PlaceholderKind::Value(1))]
        );
        assert_eq!(base_ast.bind_index, 1);
    }

    #[test]
    fn emit_placeholders_with_many_columns_emits_comma_separated_placeholders() {
        let columns = vec![make_column("id"), make_column("name"), make_column("email")];
        let mut base_ast = make_base_ast();
        let mut tokens = SqlTokens::default();

        emit_placeholders(&columns, &mut base_ast, &mut tokens);

        assert_eq!(tokens.inner(), get_placeholders_test_expr_values_3());
        assert_eq!(base_ast.bind_index, 3);
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn emit_placeholders_with_postgres_uses_dollar_numbering() {
        let columns = vec![make_column("id"), make_column("name")];
        let mut base_ast = make_base_ast();
        let mut tokens = SqlTokens::default();

        emit_placeholders(&columns, &mut base_ast, &mut tokens);

        assert_eq!(tokens.inner(), get_placeholders_test_expr_values());
        assert_eq!(base_ast.bind_index, 2);
        assert_eq!(
            PgDialect::PLACEHOLDER_SYMBOL,
            PlaceholderSymbol::DollarNumbered
        );
    }

    #[cfg(feature = "mysql")]
    #[test]
    fn emit_placeholders_with_mysql_uses_question_marks() {
        let columns = vec![make_column("id"), make_column("name")];
        let mut base_ast = make_base_ast();
        let mut tokens = SqlTokens::default();

        emit_placeholders(&columns, &mut base_ast, &mut tokens);

        assert_eq!(tokens.inner(), get_placeholders_test_expr_values());
        assert_eq!(base_ast.bind_index, 2);
        assert_eq!(MySql::PLACEHOLDER_SYMBOL, PlaceholderSymbol::QuestionMark);
    }

    fn get_placeholders_test_expr_values() -> Vec<SqlToken<'static>> {
        vec![
            SqlToken::Placeholder(PlaceholderKind::Value(1)),
            SqlToken::Symbol(Comma),
            SqlToken::WhiteSpace,
            SqlToken::Placeholder(PlaceholderKind::Value(2)),
        ]
    }

    fn get_placeholders_test_expr_values_3() -> Vec<SqlToken<'static>> {
        let mut v = vec![];
        v.extend(get_placeholders_test_expr_values());
        v.extend(vec![
            SqlToken::Symbol(Comma),
            SqlToken::WhiteSpace,
            SqlToken::Placeholder(PlaceholderKind::Value(3)),
        ]);
        v
    }

    #[cfg(feature = "mssql")]
    #[test]
    fn emit_placeholders_with_mssql_uses_at_p_numbering() {
        let columns = vec![make_column("id"), make_column("name"), make_column("email")];
        let mut base_ast = make_base_ast();
        let mut tokens = SqlTokens::default();

        emit_placeholders(&columns, &mut base_ast, &mut tokens);

        assert_eq!(
            tokens.inner(),
            vec![
                SqlToken::Placeholder(PlaceholderKind::Value(1)),
                SqlToken::Symbol(Comma),
                SqlToken::WhiteSpace,
                SqlToken::Placeholder(PlaceholderKind::Value(2)),
                SqlToken::Symbol(Comma),
                SqlToken::WhiteSpace,
                SqlToken::Placeholder(PlaceholderKind::Value(3)),
            ]
        );
        assert_eq!(base_ast.bind_index, 3);
        assert_eq!(MsSql::PLACEHOLDER_SYMBOL, PlaceholderSymbol::AtPNumbered);
    }

    #[test]
    fn emit_placeholders_respects_existing_bind_index_offset() {
        let columns = vec![make_column("id"), make_column("name")];
        let mut base_ast = make_base_ast();
        base_ast.bind_index = 4;
        let mut tokens = SqlTokens::default();

        emit_placeholders(&columns, &mut base_ast, &mut tokens);

        assert_eq!(
            tokens.inner(),
            vec![
                SqlToken::Placeholder(PlaceholderKind::Value(5)),
                SqlToken::Symbol(Comma),
                SqlToken::WhiteSpace,
                SqlToken::Placeholder(PlaceholderKind::Value(6)),
            ]
        );
        assert_eq!(base_ast.bind_index, 6);
    }

    #[test]
    fn emit_placeholders_can_continue_from_previous_state() {
        let columns = vec![make_column("id"), make_column("name")];
        let mut base_ast = make_base_ast();
        let mut first = SqlTokens::default();
        let mut second = SqlTokens::default();

        emit_placeholders(&columns, &mut base_ast, &mut first);
        emit_placeholders(&columns, &mut base_ast, &mut second);

        assert_eq!(
            first.inner(),
            vec![
                SqlToken::Placeholder(PlaceholderKind::Value(1)),
                SqlToken::Symbol(Comma),
                SqlToken::WhiteSpace,
                SqlToken::Placeholder(PlaceholderKind::Value(2)),
            ]
        );
        assert_eq!(
            second.inner(),
            vec![
                SqlToken::Placeholder(PlaceholderKind::Value(3)),
                SqlToken::Symbol(Comma),
                SqlToken::WhiteSpace,
                SqlToken::Placeholder(PlaceholderKind::Value(4)),
            ]
        );
        assert_eq!(base_ast.bind_index, 4);
    }

    #[test]
    fn base_ast_new_starts_bind_index_at_one() {
        let ast = BaseAst::new("users");

        assert_eq!(ast.bind_index, 1);
        assert_eq!(ast.table.to_string(), "users");
        assert!(ast.conditions.is_empty());
    }

    #[test]
    fn next_placeholder_index_increments_and_returns_new_index() {
        let mut ast = BaseAst::new("users");

        let first = ast.next_placeholder_index();
        let second = ast.next_placeholder_index();

        assert_eq!(first, 2);
        assert_eq!(second, 3);
        assert_eq!(ast.bind_index, 3);
    }
}
