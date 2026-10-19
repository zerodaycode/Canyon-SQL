use crate::connection::database_type::DatabaseType;
use std::fmt::Display;

/// Governs syntax rules such as placeholder format,
/// quoting style, and supported clauses.
/// For example, PostgreSQL may allow `RETURNING`, while MySQL does not.
///
/// Default values are set to the most common and widely supported syntax, which is
/// the ANSI SQL standard. Specific dialects can override these defaults as needed.
pub trait SqlDialect {
    const DB: DatabaseType;
    const SUPPORTS_RETURNING: bool = true;
    const _SUPPORTS_LIMIT_OFFSET: bool = true; // TODO: pending to implement
    const IDENT_QUOTING: IdentQuotingStyle = IdentQuotingStyle::DoubleQuote;
    const PLACEHOLDER_SYMBOL: PlaceholderSymbol = PlaceholderSymbol::DollarNumbered;
    const PLACEHOLDER_DATA_TYPE: PlaceholderDatatype = PlaceholderDatatype::Varchar;
}

#[cfg(feature = "postgres")]
pub struct PgDialect;
#[cfg(feature = "postgres")]
impl SqlDialect for PgDialect {
    const DB: DatabaseType = DatabaseType::PostgreSql;
    const IDENT_QUOTING: IdentQuotingStyle = IdentQuotingStyle::DoubleQuote;

    const PLACEHOLDER_SYMBOL: PlaceholderSymbol = PlaceholderSymbol::DollarNumbered;
}

#[cfg(feature = "mssql")]
pub struct MsSql;
#[cfg(feature = "mssql")]
impl SqlDialect for MsSql {
    const DB: DatabaseType = DatabaseType::SqlServer;
    const IDENT_QUOTING: IdentQuotingStyle = IdentQuotingStyle::Bracket;
    const PLACEHOLDER_SYMBOL: PlaceholderSymbol = PlaceholderSymbol::AtPNumbered;
}

#[cfg(feature = "mysql")]
pub struct MySql;
#[cfg(feature = "mysql")]
impl SqlDialect for MySql {
    const DB: DatabaseType = DatabaseType::MySQL;
    const SUPPORTS_RETURNING: bool = false;
    const IDENT_QUOTING: IdentQuotingStyle = IdentQuotingStyle::Backtick;
    const PLACEHOLDER_SYMBOL: PlaceholderSymbol = PlaceholderSymbol::QuestionMark;
    const PLACEHOLDER_DATA_TYPE: PlaceholderDatatype = PlaceholderDatatype::Char;
}

/// Identifier quoting strategy for a SQL dialect.
///
/// This is intentionally small and purely syntactic: it only describes how
/// table names, column names, schema names, aliases, etc. must be delimited
/// when quoting is required.
///
/// Backend mapping:
/// - ANSI / generic SQL: `"ident"`
/// - PostgreSQL: `"ident"`
/// - MySQL: `` `ident` ``
/// - SQL Server: `[ident]`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentQuotingStyle {
    /// ANSI SQL style, used by PostgreSQL and as the generic default.
    DoubleQuote,
    /// MySQL style.
    Backtick,
    /// SQL Server style.
    Bracket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentQuoting {
    DoubleQuote,
    Backtick,
    OpeningBracket,
    ClosingBracket,
}

impl IdentQuotingStyle {
    #[inline]
    pub const fn opening(self) -> IdentQuoting {
        match self {
            Self::DoubleQuote => IdentQuoting::DoubleQuote,
            Self::Backtick => IdentQuoting::Backtick,
            Self::Bracket => IdentQuoting::OpeningBracket,
        }
    }

    #[inline]
    pub const fn closing(self) -> IdentQuoting {
        match self {
            Self::DoubleQuote => IdentQuoting::DoubleQuote,
            Self::Backtick => IdentQuoting::Backtick,
            Self::Bracket => IdentQuoting::ClosingBracket,
        }
    }

    /// The identifier with this style's closing delimiter doubled, ready to go between
    /// the delimiters. Borrowed when there is nothing to escape.
    pub fn escape<'a>(
        self,
        ident: impl Into<std::borrow::Cow<'a, str>>,
    ) -> std::borrow::Cow<'a, str> {
        let ident = ident.into();
        let close = self.closing().to_string();
        if ident.contains(&close) {
            std::borrow::Cow::Owned(ident.replace(&close, &close.repeat(2)))
        } else {
            ident
        }
    }

    /// The identifier between this style's delimiters, with the closing one doubled.
    pub fn quote(self, ident: &str) -> String {
        format!("{}{}{}", self.opening(), self.escape(ident), self.closing())
    }
}

impl Display for IdentQuoting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = match self {
            Self::DoubleQuote => "\"",
            Self::Backtick => "`",
            Self::OpeningBracket => "[",
            Self::ClosingBracket => "]",
        };
        write!(f, "{}", t)
    }
}

/// Represents the syntax style for parameter placeholders in prepared statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderSymbol {
    /// ?, ?, ?
    QuestionMark,
    /// $1, $2, $3
    DollarNumbered,
    /// @p1, @p2, @p3
    AtPNumbered,
    /// :1, :2, :3
    _ColonNumbered,
}

impl Display for PlaceholderSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Self::QuestionMark => "?",
            Self::DollarNumbered => "$",
            Self::AtPNumbered => "@P",
            Self::_ColonNumbered => ":",
        };
        write!(f, "{}", symbol)
    }
}

/// Represents the syntax style for parameter placeholders in prepared statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderDatatype {
    Varchar,
    Char,
}

impl From<PlaceholderDatatype> for &'static str {
    fn from(datatype: PlaceholderDatatype) -> Self {
        match datatype {
            PlaceholderDatatype::Varchar => "VARCHAR",
            PlaceholderDatatype::Char => "CHAR",
        }
    }
}

impl Display for PlaceholderDatatype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let datatype = match self {
            Self::Varchar => "VARCHAR",
            Self::Char => "CHAR",
        };
        write!(f, "{}", datatype)
    }
}

#[cfg(test)]
mod quote_tests {
    use super::IdentQuotingStyle;

    #[test]
    fn quote_wraps_and_escapes_the_closing_delimiter() {
        assert_eq!(IdentQuotingStyle::DoubleQuote.quote("a\"b"), "\"a\"\"b\"");
        assert_eq!(IdentQuotingStyle::Backtick.quote("a`b"), "`a``b`");
        assert_eq!(IdentQuotingStyle::Bracket.quote("a]b"), "[a]]b]");
        assert_eq!(IdentQuotingStyle::Bracket.quote("plain"), "[plain]");
    }
}
