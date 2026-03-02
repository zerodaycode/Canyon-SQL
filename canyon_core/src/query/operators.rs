use crate::connection::database_type::DatabaseType;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::tokens::{SqlToken, SqlTokens, Symbol, ToSqlTokens};
use std::fmt::Display;

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
            Self::Like(ref __kind) => "LIKE",
        };
        write!(f, "{}", op)
    }
}

impl<'a> ToSqlTokens<'a> for Comp {
    fn to_tokens(&self, out: &mut SqlTokens<'a>) {
        match *self {
            Comp::Eq => out.symbol(Symbol::Equals),
            Comp::Neq => {
                out.symbol(Symbol::Not);
                out.symbol(Symbol::Equals)
            }
            Comp::Gt => out.symbol(Symbol::RAngle),
            Comp::GtEq => {
                out.symbol(Symbol::RAngle);
                out.symbol(Symbol::Equals)
            }
            Comp::Lt => out.symbol(Symbol::LAngle),
            Comp::LtEq => {
                out.symbol(Symbol::LAngle);
                out.symbol(Symbol::Equals)
            }
            Comp::Like(__kind) => out.keyword(Keyword::Like),
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
    pub(crate) fn as_str(
        &self,
        placeholder_counter: usize,
        datasource_type: DatabaseType,
    ) -> String {
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
