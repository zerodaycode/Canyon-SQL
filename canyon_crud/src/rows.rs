use crate::crud::Transaction;
use crate::mapper::RowMapper;
use std::marker::PhantomData;

/// Lightweight wrapper over the collection of results of the different crates
/// supported by Canyon-SQL.
///
/// Even tho the wrapping seems meaningless, this allows us to provide internal
/// operations that are too difficult or to ugly to implement in the macros that
/// will call the query method of Crud.
pub enum CanyonRows<T> {
    #[cfg(feature = "postgres")]
    Postgres(Vec<tokio_postgres::Row>),
    #[cfg(feature = "mssql")]
    Tiberius(Vec<tiberius::Row>),
    #[cfg(feature = "mysql")]
    MySQL(Vec<mysql::CanyonRowMysql>),

    UnusableTypeMarker(PhantomData<T>),
}

impl<T> CanyonRows<T> {
    #[cfg(feature = "postgres")]
    pub fn get_postgres_rows(&self) -> &Vec<tokio_postgres::Row> {
        match self {
            Self::Postgres(v) => v,
            _ => panic!("This branch will never ever should be reachable"),
        }
    }

    #[cfg(feature = "mssql")]
    pub fn get_tiberius_rows(&self) -> &Vec<tiberius::Row> {
        match self {
            Self::Tiberius(v) => v,
            _ => panic!("This branch will never ever should be reachable"),
        }
    }

    #[cfg(feature = "mysql")]
    pub fn get_mysql_rows(&self) -> &Vec<mysql::CanyonRowMysql> {
        match self {
            Self::MySQL(v) => v,
            _ => panic!("This branch will never ever should be reachable"),
        }
    }

    /// Consumes `self` and returns the wrapped [`std::vec::Vec`] with the instances of T
    pub fn into_results<Z: RowMapper<T>>(self) -> Vec<T>
    where
        T: Transaction<T>,
    {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => v.iter().map(|row| Z::deserialize_postgresql(row)).collect(),
            #[cfg(feature = "mssql")]
            Self::Tiberius(v) => v.iter().map(|row| Z::deserialize_sqlserver(row)).collect(),
            #[cfg(feature = "mysql")]
            Self::MySQL(v) => v.iter().map(|row| Z::deserialize_mysql(row)).collect(),
            _ => panic!("This branch will never ever should be reachable"),
        }
    }

    /// Returns the number of elements present on the wrapped collection
    pub fn len(&self) -> usize {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => v.len(),
            #[cfg(feature = "mssql")]
            Self::Tiberius(v) => v.len(),
            #[cfg(feature = "mysql")]
            Self::MySQL(v) => v.len(),
            _ => panic!("This branch will never ever should be reachable"),
        }
    }

    /// Returns true whenever the wrapped collection of Rows does not contains any elements
    pub fn is_empty(&self) -> bool {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(v) => v.is_empty(),
            #[cfg(feature = "mssql")]
            Self::Tiberius(v) => v.is_empty(),
            #[cfg(feature = "mysql")]
            Self::MySQL(v) => v.is_empty(),
            _ => panic!("This branch will never ever should be reachable"),
        }
    }
}

#[cfg(feature = "mysql")]
pub mod mysql {
    use mysql_async::{from_value, Column, Value};
    use mysql_common::{prelude::FromValue, row::ColumnIndex, Row};
    use std::{ops::Index, sync::Arc};

    #[derive(Debug)]
    pub struct CanyonRowMysql {
        values: Vec<Option<Value>>,
        columns: Arc<[Column]>,
    }

    impl CanyonRowMysql {
        pub fn new(values: Vec<Option<Value>>, columns: Arc<[Column]>) -> Self {
            Self { values, columns }
        }

        pub fn get<T, I>(&self, index: I) -> Option<T>
        where
            T: FromValue,
            I: ColumnIndex,
        {
            index.idx(&self.columns).and_then(|idx| {
                self.values
                    .get(idx)
                    .and_then(|x| x.as_ref())
                    .map(|x| from_value::<T>(x.clone()))
            })
        }
        pub fn get_by_index<T>(&self, index: usize) -> Option<T>
        where
            T: FromValue,
            //I: ColumnIndex,
        {
            //TODO
            self.values
                .get(index)
                .and_then(|x| x.as_ref())
                .map(|v| from_value::<T>(v.clone()))
        }
    }

    impl From<Row> for CanyonRowMysql {
        fn from(value: Row) -> Self {
            Self {
                values: value
                    .columns()
                    .iter()
                    .map(|c| value.get(c.name_str().as_ref())) //TODO
                    .collect(),
                columns: value.columns(),
            }
        }
    }

    impl Index<usize> for CanyonRowMysql {
        type Output = Value;

        fn index(&self, index: usize) -> &Value {
            self.values[index].as_ref().unwrap()
        }
    }

    impl<'a> Index<&'a str> for CanyonRowMysql {
        type Output = Value;

        fn index<'r>(&'r self, index: &'a str) -> &'r Value {
            for (i, column) in self.columns.iter().enumerate() {
                if column.name_ref() == index.as_bytes() {
                    return self.values[i].as_ref().unwrap();
                }
            }
            panic!("No such column: `{}` in row {:?}", index, self);
        }
    }
}
