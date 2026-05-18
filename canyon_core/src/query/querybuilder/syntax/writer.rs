use std::borrow::Cow;
use crate::query::ColumnRef;
use crate::query::querybuilder::syntax::symbol::Symbol;
use crate::query::querybuilder::syntax::tokens::SqlToken;
use crate::query::querybuilder::syntax::{emitter::SqlEmitter, tokens::SqlTokens};
use crate::query::querybuilder::syntax::dialect::{MsSql, SqlDialect};
use crate::query::querybuilder::syntax::emitter::types::helpers::{emit_columns, push_quoted_ident};

pub struct TokenWriter {}

impl TokenWriter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<'a, E: SqlEmitter<'a>>(
        self,
        tokens: &'a mut SqlTokens<'a>,
    ) -> Result<String, std::fmt::Error> {
        let mut out = String::new();

        if let Some(SqlToken::WhiteSpace) = tokens.last() {
            tokens.remove_last_if(|t| t.eq(&SqlToken::WhiteSpace));
        }

        tokens.symbol(Symbol::Semicolon);

        for tok in tokens {
            __impl::output_token_to_string_buffer::<E::Dialect>(tok, &mut out)?;
        }

        Ok(out.to_string())
    }
}

mod __impl {
    use crate::query::querybuilder::syntax::{
        dialect::SqlDialect, tokens::SqlToken, writer::__detail,
    };
    use std::fmt::Write;

    pub(crate) fn output_token_to_string_buffer<D: SqlDialect>(
        token: &SqlToken,
        f: &mut String,
    ) -> Result<(), std::fmt::Error> {
        let _: () = match token {
            SqlToken::Keyword(s) => write!(f, "{} ", s)?,
            SqlToken::Ident(s) => write!(f, "{}", s)?,
            SqlToken::Symbol(sym) => __detail::render_symbol(sym, f)?,
            SqlToken::Operator(op) => write!(f, "{}", op)?,
            SqlToken::Placeholder(ph_kind) => __detail::render_placeholder::<D>(ph_kind, f)?,
            SqlToken::WhiteSpace => write!(f, " ")?,
            SqlToken::Number(num) => write!(f, "{}", num)?,
        };
        Ok(())
    }
}

mod __detail {
    use crate::{
        connection::database_type::DatabaseType,
        query::querybuilder::syntax::{
            dialect::SqlDialect, symbol::Symbol, tokens::PlaceholderKind,
        },
    };
    use std::fmt::Write;

    pub(crate) fn render_symbol(sym: &Symbol, f: &mut String) -> Result<(), std::fmt::Error> {
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

    pub(crate) fn render_placeholder<D: SqlDialect>(
        ph_kind: &PlaceholderKind,
        f: &mut String,
    ) -> Result<(), std::fmt::Error> {
        match ph_kind {
            PlaceholderKind::Value(v) => write_value_placeholder::<D>(*v, f),
            PlaceholderKind::Like(like_kind, v) => write!(f, "{}", like_kind.as_str::<D>(*v)),
            PlaceholderKind::Range(start, end) => {
                write!(
                    f,
                    "({})",
                    generate_range_of_placeholders::<D>(*start, *end)?
                )
            }
        }?;
        Ok(())
    }

    fn generate_range_of_placeholders<D: SqlDialect>(
        start: usize,
        end: usize,
    ) -> Result<String, std::fmt::Error> {
        let capacity = match D::DB {
            DatabaseType::MySQL => 1,
            _ => 2,
        } * end; // TODO: custom struct to ensure that the range is correct for computing the capacity?
        let mut out_buffer = String::with_capacity(capacity);

        let mut iter = (start..end).peekable();

        while let Some(idx) = iter.next() {
            write_value_placeholder::<D>(idx, &mut out_buffer)?;

            if iter.peek().is_some() {
                write!(&mut out_buffer, ", ")?;
            }
        }

        Ok(out_buffer)
    }

    fn write_value_placeholder<D: SqlDialect>(
        idx_value: usize,
        f: &mut String,
    ) -> Result<(), std::fmt::Error> {
        let placeholder_symbol = D::PLACEHOLDER_SYMBOL;
        match D::DB {
            DatabaseType::MySQL => write!(f, "{}", placeholder_symbol),
            _ => write!(f, "{}{}", placeholder_symbol, idx_value),
        }
    }
}

mod tests {
    use super::*;

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
        push_quoted_ident::<MsSql>("users", &mut tokens);

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
        let columns = vec![ColumnRef::from("id"), ColumnRef::from("name"), ColumnRef::from("email")];
        let mut tokens = SqlTokens::default();

        emit_columns::<MsSql>(&columns, &mut tokens);

        assert_eq!(
            tokens.inner(),
            vec![
                SqlToken::Symbol(Symbol::LBracket),
                SqlToken::Ident(Cow::Borrowed("id")),
                SqlToken::Symbol(Symbol::RBracket),
                SqlToken::Symbol(Symbol::Comma),
                SqlToken::WhiteSpace,
                SqlToken::Symbol(Symbol::LBracket),
                SqlToken::Ident(Cow::Borrowed("name")),
                SqlToken::Symbol(Symbol::RBracket),
                SqlToken::Symbol(Symbol::Comma),
                SqlToken::WhiteSpace,
                SqlToken::Symbol(Symbol::LBracket),
                SqlToken::Ident(Cow::Borrowed("email")),
                SqlToken::Symbol(Symbol::RBracket),
            ]
        );
    }
}