#[cfg(feature = "mysql")]
use mysql_async::{self, prelude::ToValue};
#[cfg(feature = "mssql")]
use tiberius::{self, ColumnData, IntoSql};
#[cfg(feature = "postgres")]
use tokio_postgres::{self, types::ToSql};

// TODO: cfg all
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};

/// Defines a trait for represent type bounds against the allowed
/// data types supported by Canyon to be used as query parameters.
pub trait QueryParameter<'a>: std::fmt::Debug + Sync + Send {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync);
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_>;
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue;
}

/// The implementation of the [`canyon_connection::tiberius`] [`IntoSql`] for the
/// query parameters.
///
/// This implementation is necessary because of the generic amplitude
/// of the arguments of the [`Transaction::query`], that should work with
/// a collection of [`QueryParameter<'a>`], in order to allow a workflow
/// that is not dependent of the specific type of the argument that holds
/// the query parameters of the database connectors
#[cfg(feature = "mssql")]
impl<'a> IntoSql<'a> for &'a dyn QueryParameter<'a> {
    fn into_sql(self) -> ColumnData<'a> {
        self.as_sqlserver_param()
    }
}

//TODO Pending to review and see if it is necessary to apply something similar to the previous implementation.

impl<'a> QueryParameter<'a> for bool {
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

impl<'a> QueryParameter<'a> for i16 {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I16(Some(*self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for &i16 {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I16(Some(**self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<i16> {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I16(*self)
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<&i16> {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I16(Some(*self.unwrap()))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for i32 {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I32(Some(*self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for &i32 {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I32(Some(**self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<i32> {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I32(*self)
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<&i32> {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I32(Some(*self.unwrap()))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for f32 {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F32(Some(*self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for &f32 {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F32(Some(**self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<f32> {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F32(*self)
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<&f32> {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F32(Some(
            *self.expect("Error on an f32 value on QueryParameter<'_>"),
        ))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for f64 {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F64(Some(*self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for &f64 {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F64(Some(**self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<f64> {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F64(*self)
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<&f64> {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F64(Some(
            *self.expect("Error on an f64 value on QueryParameter<'_>"),
        ))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for i64 {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I64(Some(*self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for &i64 {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I64(Some(**self))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<i64> {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I64(*self)
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<&i64> {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I64(Some(*self.unwrap()))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for String {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::String(Some(std::borrow::Cow::Owned(self.to_owned())))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for &String {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::String(Some(std::borrow::Cow::Borrowed(self)))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<String> {
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
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<&String> {
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
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for &'_ str {
    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::String(Some(std::borrow::Cow::Borrowed(*self)))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> &dyn mysql_async::prelude::ToValue {
        self
    }
}

impl<'a> QueryParameter<'a> for Option<&'_ str> {
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

impl<'a> QueryParameter<'a> for NaiveDate {
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

impl<'a> QueryParameter<'a> for Option<NaiveDate> {
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

impl<'a> QueryParameter<'a> for NaiveTime {
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

impl<'a> QueryParameter<'a> for Option<NaiveTime> {
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

impl<'a> QueryParameter<'a> for NaiveDateTime {
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

impl<'a> QueryParameter<'a> for Option<NaiveDateTime> {
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

//TODO pending
impl<'a> QueryParameter<'a> for DateTime<FixedOffset> {
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
        todo!()
    }
}

impl<'a> QueryParameter<'a> for Option<DateTime<FixedOffset>> {
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
        todo!()
    }
}

impl<'a> QueryParameter<'a> for DateTime<Utc> {
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
        todo!()
    }
}

impl<'a> QueryParameter<'a> for Option<DateTime<Utc>> {
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
        todo!()
    }
}
