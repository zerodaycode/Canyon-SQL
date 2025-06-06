#[cfg(feature = "mysql")]
use mysql_async::{self, prelude::ToValue};
use std::any::Any;
use std::fmt::Debug;
#[cfg(feature = "mssql")]
use tiberius::{self, ColumnData, IntoSql};
#[cfg(feature = "postgres")]
use tokio_postgres::{self, types::ToSql};

// TODO: cfg feature for this re-exports, as date-time or something
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};

pub trait QueryParameterValue<'a> {
    fn downcast_ref<T: 'static>(&'a self) -> Option<&'a T>;
    fn to_owned_any<T: Clone + 'a + 'static>(&'a self) -> Box<T>;
}
impl<'a> QueryParameterValue<'a> for dyn QueryParameter {
    fn downcast_ref<T: 'static>(&'a self) -> Option<&'a T> {
        self.as_any().downcast_ref()
    }

    fn to_owned_any<T: Clone + 'a + 'static>(&'a self) -> Box<T> {
        Box::new(self.downcast_ref::<T>().cloned().unwrap())
    }
}
impl<'a> QueryParameterValue<'a> for &'a dyn QueryParameter {
    fn downcast_ref<T: 'static>(&'a self) -> Option<&'a T> {
        self.as_any().downcast_ref()
    }

    fn to_owned_any<T>(&self) -> Box<T> {
        todo!()
    }
}

// Define a zero-sized type to represent the absence of a primary key
// #[derive(Debug, Clone, Copy)]
// pub struct NoPrimaryKey;
//
// // Implement the QueryParameter trait for the zero-sized type
// impl QueryParameter for NoPrimaryKey {
//     fn as_any(&'a self) -> &'a dyn Any {
//         todo!()
//     }
//
//     fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
//         todo!()
//     }
//
//     fn as_sqlserver_param(&self) -> ColumnData<'_> {
//         todo!()
//     }
//
//     fn as_mysql_param(&self) -> &dyn ToValue {
//         todo!()
//     }
// }
//

/// Defines a trait for represent type bounds against the allowed
/// data types supported by Canyon to be used as query parameters.
pub trait QueryParameter: Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync);
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_>;
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue;
}

/// The implementation of the [`crate::connection::tiberius`] [`IntoSql`] for the
/// query parameters.
///
/// This implementation is necessary because of the generic amplitude
/// of the arguments of the [`crate::transaction::Transaction::query`], that should work with
/// a collection of [`QueryParameter`], in order to allow a workflow
/// that is not dependent of the specific type of the argument that holds
/// the query parameters of the database connectors
#[cfg(feature = "mssql")]
impl<'b> IntoSql<'b> for &'b dyn QueryParameter {
    fn into_sql(self) -> ColumnData<'b> {
        self.as_sqlserver_param()
    }
}

//TODO Pending to review and see if it is necessary to apply something similar to the previous implementation.

impl QueryParameter for bool {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::Bit(Some(*self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for i16 {
    fn as_any(&'_ self) -> &'_ dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I16(Option::from(*self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for Option<&'static i16> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I16(Some(*self.unwrap()))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for i32 {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I32(Some(*self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for Option<i32> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I32(*self)
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for u32 {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        panic!("Unsupported sqlserver parameter type <u32>");
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for Option<u32> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        panic!("Unsupported sqlserver parameter type <u32>");
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for f32 {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F32(Some(*self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for Option<f32> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F32(*self)
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for f64 {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F64(Some(*self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for Option<f64> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F64(*self)
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for i64 {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I64(Some(*self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for Option<i64> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I64(*self)
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for String {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::String(Some(std::borrow::Cow::Owned(self.to_owned())))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for Option<String> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        match self {
            Some(string) => ColumnData::String(Some(std::borrow::Cow::Owned(string.to_owned()))),
            None => ColumnData::String(None),
        }
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for Option<&'static String> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        match self {
            Some(string) => ColumnData::String(Some(std::borrow::Cow::Borrowed(string))),
            None => ColumnData::String(None),
        }
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for &'static str {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::String(Some(std::borrow::Cow::Borrowed(self)))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for Option<&'static str> {
    fn as_any(&'_ self) -> &'_ dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        match *self {
            Some(str) => ColumnData::String(Some(std::borrow::Cow::Borrowed(str))),
            None => ColumnData::String(None),
        }
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl QueryParameter for NaiveDate {
    fn as_any(&'_ self) -> &'_ dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for Option<NaiveDate> {
    fn as_any(&'_ self) -> &'_ dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for NaiveTime {
    fn as_any(&'_ self) -> &'_ dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for Option<NaiveTime> {
    fn as_any(&'_ self) -> &'_ dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

impl QueryParameter for NaiveDateTime {
    fn as_any(&'_ self) -> &'_ dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl QueryParameter for Option<NaiveDateTime> {
    fn as_any(&'_ self) -> &'_ dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        self
    }
}

//TODO pending
impl QueryParameter for DateTime<FixedOffset> {
    fn as_any(&'_ self) -> &'_ dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        todo!()
    }
}

impl QueryParameter for Option<DateTime<FixedOffset>> {
    fn as_any(&'_ self) -> &'_ dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        todo!()
    }
}

impl QueryParameter for DateTime<Utc> {
    fn as_any(&'_ self) -> &'_ dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        todo!()
    }
}

impl QueryParameter for Option<DateTime<Utc>> {
    fn as_any(&'_ self) -> &'_ dyn Any {
        self
    }

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn ToValue {
        todo!()
    }
}
