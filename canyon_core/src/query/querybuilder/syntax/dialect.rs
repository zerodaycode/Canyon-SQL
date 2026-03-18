use crate::connection::database_type::DatabaseType;
use crate::connection::database_type::DatabaseType::{MySQL, PostgreSql, SqlServer};
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
    const _SUPPORTS_LIMIT_OFFSET: bool = true;
    const IDENT_QUOTING: IdentQuoting = IdentQuoting::DoubleQuote;
    const PLACEHOLDER_SYMBOL: PlaceholderSymbol = PlaceholderSymbol::DollarNumbered;
    const PLACEHOLDER_DATA_TYPE: PlaceholderDatatype = PlaceholderDatatype::VARCHAR;
}

/// Safe assuming that Canyon's default is PostgreSQL,
/// which is the most widely used and standards-compliant database among the supported ones.
pub struct StandardDialect;
impl SqlDialect for StandardDialect {
    const DB: DatabaseType = PostgreSql;
}

#[cfg(feature = "postgres")]
pub struct PgDialect;
#[cfg(feature = "postgres")]
impl SqlDialect for PgDialect {
    const DB: DatabaseType = PostgreSql;
    const IDENT_QUOTING: IdentQuoting = IdentQuoting::DoubleQuote;
    const PLACEHOLDER_SYMBOL: PlaceholderSymbol = PlaceholderSymbol::DollarNumbered;
}

#[cfg(feature = "mssql")]
pub struct MsSql;
#[cfg(feature = "mssql")]
impl SqlDialect for MsSql {
    const DB: DatabaseType = SqlServer;
    const SUPPORTS_RETURNING: bool = false;
    const IDENT_QUOTING: IdentQuoting = IdentQuoting::Bracket;
    const PLACEHOLDER_SYMBOL: PlaceholderSymbol = PlaceholderSymbol::AtPNumbered;
}

#[cfg(feature = "mysql")]
pub struct MySql;
#[cfg(feature = "mysql")]
impl SqlDialect for MySql {
    const DB: DatabaseType = MySQL;
    const IDENT_QUOTING: IdentQuoting = IdentQuoting::Backtick;
    const PLACEHOLDER_SYMBOL: PlaceholderSymbol = PlaceholderSymbol::QuestionMark;
    const PLACEHOLDER_DATA_TYPE: PlaceholderDatatype = PlaceholderDatatype::CHAR;
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
pub enum IdentQuoting {
    /// ANSI SQL style, used by PostgreSQL and as the generic default.
    DoubleQuote,
    /// MySQL style.
    Backtick,
    /// SQL Server style.
    Bracket,
}

impl IdentQuoting {
    #[inline]
    pub const fn opening(self) -> &'static str {
        match self {
            Self::DoubleQuote => "\"",
            Self::Backtick => "`",
            Self::Bracket => "[",
        }
    }

    #[inline]
    pub const fn closing(self) -> &'static str {
        match self {
            Self::DoubleQuote => "\"",
            Self::Backtick => "`",
            Self::Bracket => "]",
        }
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
    ColonNumbered,
}

impl Display for PlaceholderSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Self::QuestionMark => "?",
            Self::DollarNumbered => "$",
            Self::AtPNumbered => "@P",
            Self::ColonNumbered => ":",
        };
        write!(f, "{}", symbol)
    }
}

/// Represents the syntax style for parameter placeholders in prepared statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderDatatype {
    VARCHAR,
    CHAR,
}

impl Display for PlaceholderDatatype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let datatype = match self {
            Self::VARCHAR => "VARCHAR",
            Self::CHAR => "CHAR",
        };
        write!(f, "{}", datatype)
    }
}
