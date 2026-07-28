use crate::query::querybuilder::syntax::{
    dialect::SqlDialect, symbol::Symbol, tokens::SqlToken, tokens::SqlTokens,
};

pub struct TokenWriter {}

impl TokenWriter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<'a, D: SqlDialect>(
        self,
        mut tokens: SqlTokens<'a>,
    ) -> Result<String, std::fmt::Error> {
        let mut out = String::new();
        tokens.symbol(Symbol::Semicolon);

        let mut placeholder_counter = 1usize;
        let mut previous: Option<&SqlToken<'a>> = None;

        for token in tokens.iter() {
            if __impl::requires_space_between(previous, token) {
                out.push(' ');
            }

            __impl::output_token_to_string_buffer::<D>(token, &mut out, &mut placeholder_counter)?;

            previous = Some(token);
        }

        Ok(out)
    }
}

mod __impl {
    use crate::query::querybuilder::syntax::{
        dialect::SqlDialect, symbol::Symbol, tokens::SqlToken, writer::__detail,
    };
    use std::fmt::Write;

    pub(crate) fn output_token_to_string_buffer<D: SqlDialect>(
        token: &SqlToken<'_>,
        f: &mut String,
        placeholder_counter: &mut usize,
    ) -> Result<(), std::fmt::Error> {
        let _: () = match token {
            SqlToken::Keyword(s) => write!(f, "{}", s)?,
            SqlToken::Ident(s) => write!(f, "{}", s)?,
            SqlToken::Symbol(sym) => __detail::render_symbol(*sym, f)?,
            SqlToken::Operator(op) => write!(f, "{}", op)?,
            SqlToken::Placeholder => {
                __detail::write_value_placeholder::<D>(placeholder_counter, f)?
            }
            SqlToken::Number(num) => write!(f, "{}", num)?,
        };
        Ok(())
    }

    pub(crate) fn requires_space_between(
        previous: Option<&SqlToken<'_>>,
        current: &SqlToken<'_>,
    ) -> bool {
        let Some(previous) = previous else {
            return false;
        };

        if is_quoted_ident_boundary(previous, current) || suppresses_trailing_space(previous) {
            return false;
        }

        wants_leading_space_after(previous, current)
    }

    fn wants_leading_space_after(_previous: &SqlToken<'_>, current: &SqlToken<'_>) -> bool {
        matches!(
            current,
            SqlToken::Keyword(_)
                | SqlToken::Ident(_)
                | SqlToken::Number(_)
                | SqlToken::Placeholder
                | SqlToken::Operator(_)
                | SqlToken::Symbol(
                    Symbol::Asterisk
                        | Symbol::LParen
                        | Symbol::Quote
                        | Symbol::DoubleQuote
                        | Symbol::Backtick
                        | Symbol::LBracket
                        | Symbol::Equals
                )
        )
    }

    fn suppresses_trailing_space(token: &SqlToken<'_>) -> bool {
        matches!(
            token,
            SqlToken::Symbol(
                Symbol::Dot
                    | Symbol::LParen
                    | Symbol::LBracket
                    | Symbol::PercentSign
                    | Symbol::Backslash
            )
        )
    }

    fn is_quoted_ident_boundary(previous: &SqlToken<'_>, current: &SqlToken<'_>) -> bool {
        matches!(
            (previous, current),
            (
                SqlToken::Symbol(Symbol::Quote | Symbol::DoubleQuote | Symbol::Backtick),
                SqlToken::Ident(_),
            ) | (
                SqlToken::Ident(_),
                SqlToken::Symbol(Symbol::Quote | Symbol::DoubleQuote | Symbol::Backtick),
            )
        )
    }
}

mod __detail {
    use crate::{
        connection::database_type::DatabaseType,
        query::querybuilder::syntax::{dialect::SqlDialect, symbol::Symbol},
    };
    use std::fmt::Write;

    pub(crate) fn render_symbol(sym: Symbol, f: &mut String) -> Result<(), std::fmt::Error> {
        let _: () = match sym {
            Symbol::Not => write!(f, "!")?,
            Symbol::Comma => write!(f, ",")?,
            Symbol::LParen => write!(f, "(")?,
            Symbol::RParen => write!(f, ")")?,
            Symbol::Dot => write!(f, ".")?,
            Symbol::Semicolon => write!(f, ";")?,
            Symbol::Equals => write!(f, "=")?,
            Symbol::Asterisk => write!(f, "*")?,
            Symbol::Apostrophe => write!(f, "'")?,
            Symbol::LAngle => write!(f, "<")?,
            Symbol::RAngle => write!(f, ">")?,
            Symbol::PercentSign => write!(f, "%")?,
            Symbol::Quote => write!(f, "'")?,
            Symbol::DoubleQuote => write!(f, "\"")?,
            Symbol::Backtick => write!(f, "`")?,
            Symbol::LBracket => write!(f, "[")?,
            Symbol::RBracket => write!(f, "]")?,
            Symbol::Backslash => write!(f, "\\")?,
            Symbol::Empty => write!(f, "")?,
        };
        Ok(())
    }

    pub(crate) fn write_value_placeholder<D: SqlDialect>(
        placeholder_counter: &mut usize,
        f: &mut String,
    ) -> Result<(), std::fmt::Error> {
        let placeholder_symbol = D::PLACEHOLDER_SYMBOL;
        let _ = match D::DB {
            DatabaseType::MySQL => write!(f, "{}", placeholder_symbol),
            _ => write!(f, "{}{}", placeholder_symbol, placeholder_counter),
        };

        *placeholder_counter += 1;

        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "mssql")]
mod mssql_tests {
    use crate::query::ColumnRef;
    use crate::query::querybuilder::syntax::dialect::{MsSql, SqlDialect};
    use crate::query::querybuilder::syntax::emitter::types::helpers::{
        emit_qualified_columns, push_quoted_ident,
    };
    use crate::query::querybuilder::syntax::symbol::Symbol;
    use crate::query::querybuilder::syntax::tokens::{SqlToken, SqlTokens};
    use std::borrow::Cow;

    #[cfg(feature = "mssql")]
    #[test]
    fn mssql_ident_quoting_opening_and_closing_convert_to_expected_symbols() {
        use crate::query::querybuilder::syntax::symbol::Symbol;

        let opening: Symbol = MsSql::IDENT_QUOTING.opening().into();
        let closing: Symbol = MsSql::IDENT_QUOTING.closing().into();

        assert_eq!(opening, Symbol::LBracket);
        assert_eq!(closing, Symbol::RBracket);
        assert_ne!(opening, closing);
    }

    #[cfg(feature = "mssql")]
    #[test]
    fn push_quoted_ident_with_mssql_emits_left_ident_right_bracket_sequence() {
        let mut tokens = SqlTokens::default();
        push_quoted_ident::<MsSql, &str>("users", &mut tokens);

        assert_eq!(
            tokens.inner(),
            vec![
                SqlToken::Symbol(Symbol::LBracket),
                SqlToken::Ident(Cow::Borrowed("users")),
                SqlToken::Symbol(Symbol::RBracket),
            ]
        );
    }

    #[cfg(feature = "mssql")]
    #[test]
    fn emit_columns_with_mssql_emits_balanced_brackets_for_every_identifier() {
        let columns = get_columns_mock();
        let mut tokens = SqlTokens::default();
        emit_qualified_columns::<MsSql>(&columns, &mut tokens);

        assert_eq!(
            tokens.inner(),
            get_columns_assert_values(
                MsSql::IDENT_QUOTING.opening().into(),
                MsSql::IDENT_QUOTING.closing().into()
            )
        );
    }

    fn get_columns_mock() -> Vec<ColumnRef<'static>> {
        vec![
            ColumnRef::from("id"),
            ColumnRef::from("name"),
            ColumnRef::from("email"),
        ]
    }

    fn get_columns_assert_values(opening: Symbol, closing: Symbol) -> Vec<SqlToken<'static>> {
        vec![
            SqlToken::Symbol(opening),
            SqlToken::Ident(Cow::Borrowed("id")),
            SqlToken::Symbol(closing),
            SqlToken::Symbol(Symbol::Comma),
            SqlToken::Symbol(opening),
            SqlToken::Ident(Cow::Borrowed("name")),
            SqlToken::Symbol(closing),
            SqlToken::Symbol(Symbol::Comma),
            SqlToken::Symbol(opening),
            SqlToken::Ident(Cow::Borrowed("email")),
            SqlToken::Symbol(closing),
        ]
    }
}

#[cfg(test)]
mod spacing_tests {
    use super::*;
    use crate::query::operators::Operator;
    use crate::query::querybuilder::syntax::dialect::PgDialect;
    use crate::query::querybuilder::syntax::emitter::backends::PgEmitter;
    use crate::query::querybuilder::syntax::keyword::Keyword;
    use crate::query::querybuilder::syntax::tokens::SqlTokens;

    #[test]
    fn render_spaces_select_from_where_and_operators_without_whitespace_tokens() {
        let mut tokens = SqlTokens::default();
        tokens.keyword(Keyword::Select);
        tokens.symbol(Symbol::Asterisk);
        tokens.keyword(Keyword::From);
        tokens.symbol(Symbol::DoubleQuote);
        tokens.ident("league");
        tokens.symbol(Symbol::DoubleQuote);
        tokens.keyword(Keyword::Where);
        tokens.symbol(Symbol::DoubleQuote);
        tokens.ident("id");
        tokens.symbol(Symbol::DoubleQuote);
        tokens.operator(Operator::Gt);
        tokens.placeholder();

        let sql = TokenWriter::new()
            .render::<PgDialect>(tokens)
            .expect("failed to render SQL");

        assert_eq!(sql, "SELECT * FROM \"league\" WHERE \"id\" > $1;");
    }

    #[test]
    fn render_does_not_insert_spaces_inside_quoted_identifiers() {
        let mut tokens = SqlTokens::default();
        tokens.keyword(Keyword::Select);
        tokens.symbol(Symbol::DoubleQuote);
        tokens.ident("league");
        tokens.symbol(Symbol::DoubleQuote);
        tokens.symbol(Symbol::Dot);
        tokens.symbol(Symbol::DoubleQuote);
        tokens.ident("id");
        tokens.symbol(Symbol::DoubleQuote);
        tokens.keyword(Keyword::From);
        tokens.symbol(Symbol::DoubleQuote);
        tokens.ident("league");
        tokens.symbol(Symbol::DoubleQuote);

        let sql = TokenWriter::new()
            .render::<PgDialect>(tokens)
            .expect("failed to render SQL");

        assert_eq!(sql, "SELECT \"league\".\"id\" FROM \"league\";");
    }

    #[test]
    fn render_spaces_commas_function_calls_and_parentheses_without_trailing_comma_space() {
        let mut tokens = SqlTokens::default();
        tokens.keyword(Keyword::In);
        tokens.symbol(Symbol::LParen);
        tokens.placeholder();
        tokens.symbol(Symbol::Comma);
        tokens.placeholder();
        tokens.symbol(Symbol::RParen);

        let sql = TokenWriter::new()
            .render::<PgDialect>(tokens)
            .expect("failed to render SQL");

        assert_eq!(sql, "IN ($1, $2);");
    }

    #[test]
    fn render_spaces_function_call_parentheses_and_in_parentheses() {
        let mut tokens = SqlTokens::default();
        tokens.keyword(Keyword::Like);
        tokens.keyword(Keyword::Concat);
        tokens.symbol(Symbol::LParen);
        tokens.symbol(Symbol::Quote);
        tokens.symbol(Symbol::PercentSign);
        tokens.symbol(Symbol::Quote);
        tokens.symbol(Symbol::Comma);
        tokens.keyword(Keyword::Cast);
        tokens.symbol(Symbol::LParen);
        tokens.placeholder();
        tokens.keyword(Keyword::As);
        tokens.ident("VARCHAR");
        tokens.symbol(Symbol::RParen);
        tokens.symbol(Symbol::Comma);
        tokens.symbol(Symbol::Quote);
        tokens.symbol(Symbol::PercentSign);
        tokens.symbol(Symbol::Quote);
        tokens.symbol(Symbol::RParen);
        tokens.keyword(Keyword::In);
        tokens.symbol(Symbol::LParen);
        tokens.placeholder();
        tokens.symbol(Symbol::Comma);
        tokens.placeholder();
        tokens.symbol(Symbol::RParen);

        let sql = TokenWriter::new()
            .render::<PgDialect>(tokens)
            .expect("failed to render SQL");

        assert_eq!(
            sql,
            "LIKE CONCAT ('%', CAST ($1 AS VARCHAR), '%') IN ($2, $3);"
        );
    }
}
