use std::fmt::Display;
use crate::connection::database_type::DatabaseType;
use crate::query::querybuilder::syntax::tokens::{SqlToken, Symbol, ToSqlTokens};

/// Enumerated type for represent the comparison operations
/// in SQL sentences
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Comp {
    /// Operator "=" equals
    Eq,
    /// Operator "!=" not equals
    Neq,
    /// Operator ">" greater than value
    Gt,
    /// Operator ">=" greater or equals than value
    GtEq,
    /// Operator "<" less than value
    Lt,
    /// Operator "=<" less or equals than value
    LtEq,
    /// A "LIKE" comp operator
    Like(LikeKind),
}

impl Display for Comp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match *self {
            Self::Eq => "=",
            Self::Neq => "<>",
            Self::Gt => ">",
            Self::GtEq => ">=",
            Self::Lt => "<",
            Self::LtEq => "<=",
            Self::Like(ref __kind) => "LIKE"
        };
        write!(f, "{}", op)
    }
}

impl<'a> ToSqlTokens<'a> for Comp {
    fn to_tokens(&self, out: &mut Vec<SqlToken<'a>>) {
        match *self {
            Comp::Eq => out.push(SqlToken::Symbol(Symbol::Equals)),
            Comp::Neq => {
                out.push(SqlToken::Symbol(Symbol::Not));
                out.push(SqlToken::Symbol(Symbol::Equals))
            }
            Comp::Gt => out.push(SqlToken::Symbol(Symbol::RAngle)),
            Comp::GtEq => {
                out.push(SqlToken::Symbol(Symbol::RAngle));
                out.push(SqlToken::Symbol(Symbol::Equals))
            }
            Comp::Lt => out.push(SqlToken::Symbol(Symbol::LAngle)),
            Comp::LtEq => {
                out.push(SqlToken::Symbol(Symbol::LAngle));
                out.push(SqlToken::Symbol(Symbol::Equals))
            }
            Comp::Like(__kind) => out.push(SqlToken::new_keyword("LIKE"))
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LikeKind {
    /// Operator "LIKE"  as '%pattern%'
    Full,
    /// Operator "LIKE"  as '%pattern'
    Left,
    /// Operator "LIKE"  as 'pattern%'
    Right,
}

impl LikeKind {
    pub(crate) fn as_str(&self, placeholder_counter: usize, datasource_type: DatabaseType) -> String {
        let type_data_to_cast_str = match datasource_type {
            #[cfg(feature = "postgres")]
            DatabaseType::PostgreSql => "VARCHAR",
            #[cfg(feature = "mssql")]
            DatabaseType::SqlServer => "VARCHAR",
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => "CHAR",
            _ => panic!("Provisional LIKE"),
        };

        match *self {
            Self::Full => {
                format!(
                    " LIKE CONCAT('%', CAST(${placeholder_counter} AS {type_data_to_cast_str}) ,'%')"
                )
            }
            Self::Left => format!(
                " LIKE CONCAT('%', CAST(${placeholder_counter} AS {type_data_to_cast_str}))"
            ),
            Self::Right => format!(
                " LIKE CONCAT(CAST(${placeholder_counter} AS {type_data_to_cast_str}) ,'%')"
            ),
        }
    }
}
