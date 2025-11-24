use crate::connection::database_type::DatabaseType;
use std::fmt::{Display, Formatter};

pub trait Operator: Display {
    fn as_str(&self, placeholder_counter: usize, datasource_type: &DatabaseType) -> String;
}

/// Enumerated type for represent the comparison operations
/// in SQL sentences
#[derive(Debug)]
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
    Like(LikeKind)
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

#[derive(Debug)]
pub enum LikeKind {
    /// Operator "LIKE"  as '%pattern%'
    Full,
    /// Operator "LIKE"  as '%pattern'
    Left,
    /// Operator "LIKE"  as 'pattern%'
    Right,
}

impl Operator for LikeKind {
    fn as_str(&self, placeholder_counter: usize, datasource_type: &DatabaseType) -> String {
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

impl Display for LikeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::Full => "Like::Full",
                Self::Left => "Like::Left",
                Self::Right => "Like::Right",
            }
        )
    }
}
