/// Governs syntax rules such as placeholder format,
/// quoting style, and supported clauses.
/// For example, PostgreSQL may allow `RETURNING`, while MySQL does not.
pub trait SqlDialect {
    const SUPPORTS_RETURNING: bool = true;
    const SUPPORTS_LIMIT_OFFSET: bool = true;
    // const IDENT_QUOTING: IdentQuoting;
}

#[cfg(feature = "postgres")]
pub struct PgDialect;
#[cfg(feature = "postgres")]
impl SqlDialect for PgDialect {}

#[cfg(feature = "mssql")]
pub struct MsSql;
#[cfg(feature = "mssql")]
impl SqlDialect for MsSql {
    const SUPPORTS_RETURNING: bool = false;
}
#[cfg(feature = "mysql")]
pub struct MySql;
#[cfg(feature = "mysql")]
impl SqlDialect for MySql {}
