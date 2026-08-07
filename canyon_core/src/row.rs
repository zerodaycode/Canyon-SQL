#![allow(unused_imports)]

#[cfg(feature = "mysql")]
use mysql_async::{self};
#[cfg(feature = "mssql")]
use tiberius::{self};
#[cfg(feature = "postgres")]
use tokio_postgres::{self};

use crate::column::{Column, ColumnType};
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
    fn get_postgres<'a, Output>(&'a self, col_name: &'a str) -> Output
    where
        Output: tokio_postgres::types::FromSql<'a>;
    #[cfg(feature = "mssql")]
    fn get_mssql<'a, Output>(&'a self, col_name: &'a str) -> Output
    where
        Output: tiberius::FromSql<'a>;
    #[cfg(feature = "mysql")]
    fn get_mysql<'a, Output>(&'a self, col_name: &'a str) -> Output
    where
        Output: mysql_async::prelude::FromValue;

    #[cfg(feature = "postgres")]
    fn get_postgres_opt<'a, Output>(&'a self, col_name: &'a str) -> Option<Output>
    where
        Output: tokio_postgres::types::FromSql<'a>;
    #[cfg(feature = "mssql")]
    fn get_mssql_opt<'a, Output>(&'a self, col_name: &'a str) -> Option<Output>
    where
        Output: tiberius::FromSql<'a>;

    #[cfg(feature = "mysql")]
    fn get_mysql_opt<'a, Output>(&'a self, col_name: &'a str) -> Option<Output>
    where
        Output: mysql_async::prelude::FromValue;

    fn columns(&self) -> Vec<Column<'_>>;
}

impl RowOperations for &dyn Row {
    #[cfg(feature = "postgres")]
    fn get_postgres<'a, Output>(&'a self, col_name: &'a str) -> Output
    where
        Output: tokio_postgres::types::FromSql<'a>,
    {
        if let Some(row) = self.as_any().downcast_ref::<tokio_postgres::Row>() {
            return row.get::<&str, Output>(col_name);
        };
        panic!() // TODO into result and propagate
    }
    #[cfg(feature = "mssql")]
    fn get_mssql<'a, Output>(&'a self, col_name: &'a str) -> Output
    where
        Output: tiberius::FromSql<'a>,
    {
        if let Some(row) = self.as_any().downcast_ref::<tiberius::Row>() {
            return row
                .get::<Output, &str>(col_name)
                .expect("Failed to obtain a row in the MSSQL migrations");
        };
        panic!() // TODO into result and propagate
    }

    #[cfg(feature = "mysql")]
    fn get_mysql<'a, Output>(&'a self, col_name: &'a str) -> Output
    where
        Output: mysql_async::prelude::FromValue,
    {
        self.get_mysql_opt(col_name)
            .expect("Failed to obtain a column in the MySql")
    }

    #[cfg(feature = "postgres")]
    fn get_postgres_opt<'a, Output>(&'a self, col_name: &'a str) -> Option<Output>
    where
        Output: tokio_postgres::types::FromSql<'a>,
    {
        if let Some(row) = self.as_any().downcast_ref::<tokio_postgres::Row>() {
            return row.get::<&str, Option<Output>>(col_name);
        };
        panic!() // TODO into result and propagate
    }

    #[cfg(feature = "mssql")]
    fn get_mssql_opt<'a, Output>(&'a self, col_name: &'a str) -> Option<Output>
    where
        Output: tiberius::FromSql<'a>,
    {
        if let Some(row) = self.as_any().downcast_ref::<tiberius::Row>() {
            return row.get::<Output, &str>(col_name);
        };
        panic!() // TODO into result and propagate
    }
    #[cfg(feature = "mysql")]
    fn get_mysql_opt<'a, Output>(&'a self, col_name: &'a str) -> Option<Output>
    where
        Output: mysql_async::prelude::FromValue,
    {
        if let Some(row) = self.as_any().downcast_ref::<mysql_async::Row>() {
            return row.get::<Output, &str>(col_name);
        };
        panic!() // TODO into result and propagate
    }

    fn columns(&self) -> Vec<Column<'_>> {
        let mut cols = vec![];

        #[cfg(feature = "postgres")]
        {
            if self.as_any().is::<tokio_postgres::Row>() {
                self.as_any()
                    .downcast_ref::<tokio_postgres::Row>()
                    .expect("Not a tokio postgres Row for column")
                    .columns()
                    .iter()
                    .for_each(|c| {
                        cols.push(Column {
                            name: std::borrow::Cow::from(c.name()),
                            type_: crate::column::ColumnType::Postgres(c.type_().to_owned()),
                        })
                    })
            }
        }
        #[cfg(feature = "mssql")]
        {
            if self.as_any().is::<tiberius::Row>() {
                self.as_any()
                    .downcast_ref::<tiberius::Row>()
                    .expect("Not a Tiberius Row for column")
                    .columns()
                    .iter()
                    .for_each(|c| {
                        cols.push(Column {
                            name: Cow::from(c.name()),
                            type_: ColumnType::SqlServer(c.column_type()),
                        })
                    })
            };
        }
        #[cfg(feature = "mysql")]
        {
            if let Some(mysql_row) = self.as_any().downcast_ref::<mysql_async::Row>() {
                mysql_row.columns_ref().iter().for_each(|c| {
                    cols.push(Column {
                        name: c.name_str(),
                        type_: ColumnType::MySQL(c.column_type()),
                    })
                })
            }
        }

        cols
    }
}
