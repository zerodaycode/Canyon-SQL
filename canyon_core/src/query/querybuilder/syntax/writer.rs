use crate::connection::database_type::DatabaseType;
use crate::query::querybuilder::syntax::tokens::SqlToken;

pub struct TokenWriter {}

impl TokenWriter {
    pub fn new() -> Self { Self {} }

    pub fn render(
        self,
        tokens: &[SqlToken],
        db: DatabaseType,
    ) -> Result<String, std::fmt::Error> {
        let mut out = String::new();

        for tok in tokens {
            __impl::output_token_to_string_buffer(tok, db, &mut out)?; // TODO: split, for db and others (maybe)
        }

        Ok(out.trim_start().to_string())
    }
}

mod __impl {
    use crate::connection::database_type::DatabaseType;
    use crate::query::querybuilder::syntax::tokens::{SqlToken, Symbol};
    use std::fmt::Write;
    use crate::query::querybuilder::syntax::writer::__detail;

    pub(crate) fn output_token_to_string_buffer(
        token: &SqlToken,
        db: DatabaseType,
        f: &mut String,
    ) -> Result<(), std::fmt::Error> {
        let _: () = match token {
            SqlToken::Keyword(s) => write!(f, " {}", s)?,
            SqlToken::Ident(s) => write!(f, " {}", s)?,
            SqlToken::Symbol(sym) => __detail::render_symbol(sym, f)?,
            SqlToken::Operator(op) => write!(f, " {}", op)?,
            SqlToken::Placeholder(ph_kind) => __detail::render_placeholder(ph_kind, f, db)?,
            _ => {}
        };
        Ok(())
    }
}

mod __detail {
    use std::fmt::Write;
    use crate::connection::database_type::DatabaseType;
    use crate::query::querybuilder::syntax::symbol::Symbol;
    use crate::query::querybuilder::syntax::tokens::PlaceholderKind;

    pub(crate) fn render_symbol(sym: &Symbol, f: &mut String) -> Result<(), std::fmt::Error> {
        let _: () = match sym {
            Symbol::Not => write!(f, "!")?,
            Symbol::Comma => write!(f, ",")?,
            Symbol::LParen => write!(f, " (")?,
            Symbol::RParen => write!(f, ")")?,
            Symbol::Dot => write!(f, ".")?,
            Symbol::Semicolon => write!(f, ";")?,
            Symbol::Equals => write!(f, " =")?,
            Symbol::Asterisk => write!(f, " *")?,
            Symbol::Apostrophe => write!(f, "'")?,
            Symbol::LAngle => write!(f, " <")?,
            Symbol::RAngle => write!(f, " >")?,
            Symbol::PercentSign => write!(f, " %")?,
        };
        Ok(())
    }

    pub(crate) fn render_placeholder(ph_kind: &PlaceholderKind, f: &mut String, db: DatabaseType) -> Result<(), std::fmt::Error> {
        match ph_kind {
            PlaceholderKind::Value(v) => write_value_placeholder(*v, f, db),
            PlaceholderKind::Like(like_kind, v) =>
                write!(f, "{}", like_kind.as_str(*v, db)),
            PlaceholderKind::Range(start, end) =>
                write!(f, "{}", generate_range_of_placeholders(*start, *end, db)?),

        }?;
        Ok(())
    }

    fn generate_range_of_placeholders(start: usize, end: usize, db: DatabaseType) -> Result<String, std::fmt::Error> {
        let capacity = match db {
            DatabaseType::MySQL => 1,
            _ => 2
        } * end; // TODO: custom struct to ensure that the range is correct for computing the capacity?
        let mut out_buffer = String::with_capacity(capacity);

        let mut iter = (start..end).peekable();

        while let Some(idx) = iter.next() {
            write_value_placeholder(idx, &mut out_buffer, db)?;

            if iter.peek().is_some() { // Write comma *only if* there's another element coming
                write!(&mut out_buffer, ", ")?;
            }
        }

        Ok(out_buffer)
    }

    fn write_value_placeholder(idx_value: usize, f: &mut String, db: DatabaseType) -> Result<(), std::fmt::Error> {
        let placeholder_symbol = db.get_placeholder_symbol();
        match db {
            DatabaseType::SqlServer => write!(f, " {}{}", placeholder_symbol, idx_value),
            DatabaseType::MySQL => write!(f, " ?"),
            _ => write!(f, " {}{}", placeholder_symbol, idx_value),
        }
    }
}
