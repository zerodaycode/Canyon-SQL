use canyon_connection::canyon_database_connector::DatabaseType;

pub trait Operator {
    fn as_str(&self, placeholder_counter: usize, datasource_type: &DatabaseType) -> String;
}

/// Enumerated type for represent the comparison operations
/// in SQL sentences
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
}

impl Operator for Comp {
    fn as_str(&self, placeholder_counter: usize, _datasource_type: &DatabaseType) -> String {
        match *self {
            Self::Eq => format!(" = ${placeholder_counter}"),
            Self::Neq => format!(" <> ${placeholder_counter}"),
            Self::Gt => format!(" > ${placeholder_counter}"),
            Self::GtEq => format!(" >= ${placeholder_counter}"),
            Self::Lt => format!(" < ${placeholder_counter}"),
            Self::LtEq => format!(" <= ${placeholder_counter}"),
        }
    }
}

pub enum Like {
    /// Operator "LIKE"  as '%pattern%'
    Full,
    /// Operator "LIKE"  as '%pattern'
    Left,
    /// Operator "LIKE"  as 'pattern%'
    Right,
}

impl Operator for Like {
    fn as_str(&self, placeholder_counter: usize, datasource_type: &DatabaseType) -> String {
        let type_data_to_cast_str = match datasource_type {
            #[cfg(feature = "postgres")]
            DatabaseType::PostgreSql => "VARCHAR",
            #[cfg(feature = "mssql")]
            DatabaseType::SqlServer => "VARCHAR",
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => "CHAR",
        };

        match *self {
            Like::Full => {
                format!(" LIKE CONCAT('%', CAST(${placeholder_counter} AS {type_data_to_cast_str}) ,'%')")
            }
            Like::Left => format!(
                " LIKE CONCAT('%', CAST(${placeholder_counter} AS {type_data_to_cast_str}))"
            ),
            Like::Right => format!(
                " LIKE CONCAT(CAST(${placeholder_counter} AS {type_data_to_cast_str}) ,'%')"
            ),
        }
    }
}
