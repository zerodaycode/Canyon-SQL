use crate::query::querybuilder::syntax::{
    dialect::SqlDialect,
    keyword::Keyword,
    tokens::{SqlToken, SqlTokens, Symbol, ToSqlTokens},
};
use std::fmt::Display;

/// Enumerated type for represent the available operators
/// in SQL sentences
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Operator {
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
    /// Operator "IN" for value in (value1, value2, ...)
    In,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match *self {
            Self::Eq => "=",
            Self::Neq => "<>",
            Self::Gt => ">",
            Self::GtEq => ">=",
            Self::Lt => "<",
            Self::LtEq => "<=",
            Self::Like(ref __kind) => "LIKE",
            Self::In => "IN",
        };
        write!(f, "{}", op)
    }
}

impl<'a, D: SqlDialect> ToSqlTokens<'a, D> for Operator {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(2);

        match *self {
            Operator::Eq => out.symbol(Symbol::Equals),
            Operator::Neq => {
                out.symbol(Symbol::Not);
                out.symbol(Symbol::Equals)
            }
            Operator::Gt => out.symbol(Symbol::RAngle),
            Operator::GtEq => {
                out.symbol(Symbol::RAngle);
                out.symbol(Symbol::Equals)
            }
            Operator::Lt => out.symbol(Symbol::LAngle),
            Operator::LtEq => {
                out.symbol(Symbol::LAngle);
                out.symbol(Symbol::Equals)
            }
            Operator::Like(__kind) => out.keyword(Keyword::Like),
            Operator::In => out.keyword(Keyword::In),
        }

        out
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LikeKind {
    /// Operator "LIKE"  as '%pattern%'
    Full,
    /// Operator "LIKE"  as '%pattern'
    Left,
    /// Operator "LIKE"  as 'pattern%'
    Right,
}

impl LikeKind {
    pub(crate) fn as_str<D: SqlDialect>(&self, placeholder_counter: usize) -> String {
        let type_data_to_cast_str = D::PLACEHOLDER_DATA_TYPE;

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
