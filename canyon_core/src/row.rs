#![allow(unused_imports)]

#[cfg(feature = "mysql")]
use mysql_async::{self};
#[cfg(feature = "mssql")]
use tiberius::{self};
#[cfg(feature = "postgres")]
use tokio_postgres::{self};

use crate::column::{Column, ColumnType};
use crate::connection::database_type::DatabaseType;
use crate::error::{CanyonResult, MappingError};
use std::{any::Any, borrow::Cow};

/// Generic abstraction to represent any of the Row types
/// from the client crates
pub trait Row {
    fn as_any(&self) -> &dyn Any;
}

#[cfg(feature = "postgres")]
impl Row for tokio_postgres::Row {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(feature = "mssql")]
impl Row for tiberius::Row {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(feature = "mysql")]
impl Row for mysql_async::Row {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait RowOperations {
    #[cfg(feature = "postgres")]
    fn get_postgres<'a, Output>(&'a self, col_name: &'a str) -> CanyonResult<Output>
    where
        Output: tokio_postgres::types::FromSql<'a>;
    #[cfg(feature = "mssql")]
    fn get_mssql<'a, Output>(&'a self, col_name: &'a str) -> CanyonResult<Output>
    where
        Output: tiberius::FromSql<'a>;
    #[cfg(feature = "mysql")]
    fn get_mysql<Output>(&self, col_name: &str) -> CanyonResult<Output>
    where
        Output: mysql_async::prelude::FromValue;

    #[cfg(feature = "postgres")]
    fn get_postgres_opt<'a, Output>(&'a self, col_name: &'a str) -> CanyonResult<Option<Output>>
    where
        Output: tokio_postgres::types::FromSql<'a>;
    #[cfg(feature = "mssql")]
    fn get_mssql_opt<'a, Output>(&'a self, col_name: &'a str) -> CanyonResult<Option<Output>>
    where
        Output: tiberius::FromSql<'a>;

    #[cfg(feature = "mysql")]
    fn get_mysql_opt<Output>(&self, col_name: &str) -> CanyonResult<Option<Output>>
    where
        Output: mysql_async::prelude::FromValue;

    fn columns(&self) -> CanyonResult<Vec<Column<'_>>>;
}

impl RowOperations for &dyn Row {
    #[cfg(feature = "postgres")]
    fn get_postgres<'a, Output>(&'a self, col_name: &'a str) -> CanyonResult<Output>
    where
        Output: tokio_postgres::types::FromSql<'a>,
    {
        let row = self.as_any().downcast_ref::<tokio_postgres::Row>().ok_or(
            MappingError::BackendMismatch {
                expected: DatabaseType::PostgreSql,
            },
        )?;
        row.try_get::<&str, Output>(col_name)
            .map_err(|source| MappingError::postgres("<row>", col_name, source).into())
    }
    #[cfg(feature = "mssql")]
    fn get_mssql<'a, Output>(&'a self, col_name: &'a str) -> CanyonResult<Output>
    where
        Output: tiberius::FromSql<'a>,
    {
        let row =
            self.as_any()
                .downcast_ref::<tiberius::Row>()
                .ok_or(MappingError::BackendMismatch {
                    expected: DatabaseType::SqlServer,
                })?;
        row.try_get::<Output, &str>(col_name)
            .map_err(|source| MappingError::sql_server("<row>", col_name, source))?
            .ok_or_else(|| {
                MappingError::unexpected_null("<row>", col_name, DatabaseType::SqlServer).into()
            })
    }

    #[cfg(feature = "mysql")]
    fn get_mysql<Output>(&self, col_name: &str) -> CanyonResult<Output>
    where
        Output: mysql_async::prelude::FromValue,
    {
        let row = self.as_any().downcast_ref::<mysql_async::Row>().ok_or(
            MappingError::BackendMismatch {
                expected: DatabaseType::MySQL,
            },
        )?;
        row.get_opt::<Output, &str>(col_name)
            .ok_or_else(|| MappingError::column_not_found("<row>", col_name, DatabaseType::MySQL))?
            .map_err(|source| MappingError::mysql("<row>", col_name, source).into())
    }

    #[cfg(feature = "postgres")]
    fn get_postgres_opt<'a, Output>(&'a self, col_name: &'a str) -> CanyonResult<Option<Output>>
    where
        Output: tokio_postgres::types::FromSql<'a>,
    {
        let row = self.as_any().downcast_ref::<tokio_postgres::Row>().ok_or(
            MappingError::BackendMismatch {
                expected: DatabaseType::PostgreSql,
            },
        )?;
        row.try_get::<&str, Option<Output>>(col_name)
            .map_err(|source| MappingError::postgres("<row>", col_name, source).into())
    }

    #[cfg(feature = "mssql")]
    fn get_mssql_opt<'a, Output>(&'a self, col_name: &'a str) -> CanyonResult<Option<Output>>
    where
        Output: tiberius::FromSql<'a>,
    {
        let row =
            self.as_any()
                .downcast_ref::<tiberius::Row>()
                .ok_or(MappingError::BackendMismatch {
                    expected: DatabaseType::SqlServer,
                })?;
        row.try_get::<Output, &str>(col_name)
            .map_err(|source| MappingError::sql_server("<row>", col_name, source).into())
    }
    #[cfg(feature = "mysql")]
    fn get_mysql_opt<Output>(&self, col_name: &str) -> CanyonResult<Option<Output>>
    where
        Output: mysql_async::prelude::FromValue,
    {
        let row = self.as_any().downcast_ref::<mysql_async::Row>().ok_or(
            MappingError::BackendMismatch {
                expected: DatabaseType::MySQL,
            },
        )?;
        row.get_opt::<Option<Output>, &str>(col_name)
            .ok_or_else(|| MappingError::column_not_found("<row>", col_name, DatabaseType::MySQL))?
            .map_err(|source| MappingError::mysql("<row>", col_name, source).into())
    }

    fn columns(&self) -> CanyonResult<Vec<Column<'_>>> {
        let mut cols = vec![];

        #[cfg(feature = "postgres")]
        {
            if let Some(row) = self.as_any().downcast_ref::<tokio_postgres::Row>() {
                row.columns().iter().for_each(|c| {
                    cols.push(Column {
                        name: std::borrow::Cow::from(c.name()),
                        type_: crate::column::ColumnType::Postgres(c.type_().to_owned()),
                    })
                });
                return Ok(cols);
            }
        }
        #[cfg(feature = "mssql")]
        {
            if let Some(row) = self.as_any().downcast_ref::<tiberius::Row>() {
                row.columns().iter().for_each(|c| {
                    cols.push(Column {
                        name: Cow::from(c.name()),
                        type_: ColumnType::SqlServer(c.column_type()),
                    })
                });
                return Ok(cols);
            }
        }
        #[cfg(feature = "mysql")]
        {
            if let Some(mysql_row) = self.as_any().downcast_ref::<mysql_async::Row>() {
                mysql_row.columns_ref().iter().for_each(|c| {
                    cols.push(Column {
                        name: c.name_str(),
                        type_: ColumnType::MySQL(c.column_type()),
                    })
                });
                return Ok(cols);
            }
        }

        Err(MappingError::UnsupportedRowType.into())
    }
}

#[cfg(all(test, feature = "mysql"))]
mod tests {
    use super::{Row, RowOperations};
    use crate::error::{CanyonError, MappingError};
    use mysql_async::{Column, Row as MySqlRow, Value};
    use mysql_common::{constants::ColumnType, row};
    use std::sync::Arc;

    fn mysql_row(value: Value) -> MySqlRow {
        let column = Column::new(ColumnType::MYSQL_TYPE_STRING).with_name(b"value");
        row::new_row(vec![value], Arc::new([column]))
    }

    #[test]
    fn mysql_get_reports_a_missing_column() {
        let row = mysql_row(Value::Int(1));
        let row: &dyn Row = &row;

        assert!(matches!(
            row.get_mysql::<i32>("missing"),
            Err(CanyonError::Mapping(MappingError::ColumnNotFound { .. }))
        ));
    }

    #[test]
    fn mysql_get_preserves_conversion_errors() {
        let row = mysql_row(Value::Bytes(b"not-an-integer".to_vec()));
        let row: &dyn Row = &row;

        assert!(matches!(
            row.get_mysql::<i32>("value"),
            Err(CanyonError::Mapping(MappingError::MySql { .. }))
        ));
    }

    #[test]
    fn mysql_optional_get_preserves_null() {
        let row = mysql_row(Value::NULL);
        let row: &dyn Row = &row;

        assert_eq!(row.get_mysql_opt::<i32>("value").unwrap(), None);
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn getter_for_the_wrong_backend_returns_an_error() {
        let row = mysql_row(Value::Int(1));
        let row: &dyn Row = &row;

        assert!(matches!(
            row.get_postgres::<i32>("value"),
            Err(CanyonError::Mapping(MappingError::BackendMismatch { .. }))
        ));
    }
}
