/// Governs syntax rules such as placeholder format,
/// quoting style, and supported clauses.
/// For example, PostgreSQL may allow `RETURNING`, while MySQL does not.
pub trait SqlDialect {
    const SUPPORTS_RETURNING: bool = true;
    const _SUPPORTS_LIMIT_OFFSET: bool = true;
    const IDENT_QUOTING: IdentQuoting = IdentQuoting::DoubleQuote;
}

pub struct StandardDialect;
impl SqlDialect for StandardDialect {}

#[cfg(feature = "postgres")]
pub struct PgDialect;
#[cfg(feature = "postgres")]
impl SqlDialect for PgDialect {
    const IDENT_QUOTING: IdentQuoting = IdentQuoting::DoubleQuote;
}

#[cfg(feature = "mssql")]
pub struct MsSql;
#[cfg(feature = "mssql")]
impl SqlDialect for MsSql {
    const SUPPORTS_RETURNING: bool = false;
    const IDENT_QUOTING: IdentQuoting = IdentQuoting::Bracket;
}

#[cfg(feature = "mysql")]
pub struct MySql;
#[cfg(feature = "mysql")]
impl SqlDialect for MySql {
    const IDENT_QUOTING: IdentQuoting = IdentQuoting::Backtick;
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
