use std::{any::Any, borrow::Cow};

#[cfg(feature = "mysql")]
use mysql_async::{self};
#[cfg(feature = "mssql")]
use tiberius::{self};
#[cfg(feature = "postgres")]
use tokio_postgres::{self};

/// Generic abstraction for hold a Column type that will be one of the Column
/// types present in the dependent crates
// #[derive(Copy, Clone)]
pub struct Column<'a> {
    pub(crate) name: Cow<'a, str>,
    pub(crate) type_: ColumnType,
}
impl<'a> Column<'a> {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn column_type(&self) -> &ColumnType {
        &self.type_
    }
    // pub fn type_(&'a self) -> &'_ dyn Type {
    //     match (*self).type_ {
    //         #[cfg(feature = "postgres")] ColumnType::Postgres(v) => v as &'a dyn Type,
    //         #[cfg(feature = "mssql")] ColumnType::SqlServer(v) => v as &'a dyn Type,
    //     }
    // }
}

pub trait ColType {
    fn as_any(&self) -> &dyn Any;
}
#[cfg(feature = "postgres")]
impl ColType for tokio_postgres::types::Type {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[cfg(feature = "mssql")]
impl ColType for tiberius::ColumnType {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[cfg(feature = "mysql")]
impl ColType for mysql_async::consts::ColumnType {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Wrapper over the dependencies Column's types
pub enum ColumnType {
    #[cfg(feature = "postgres")]
    Postgres(tokio_postgres::types::Type),
    #[cfg(feature = "mssql")]
    SqlServer(tiberius::ColumnType),
    #[cfg(feature = "mysql")]
    MySQL(mysql_async::consts::ColumnType),
}
