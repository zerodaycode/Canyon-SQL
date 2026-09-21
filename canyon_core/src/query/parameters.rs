#[cfg(feature = "mysql")]
use mysql_async::{self, Value, prelude::ToValue};
use std::any::Any;
use std::fmt::Debug;
#[cfg(feature = "mssql")]
use tiberius::{self, ColumnData, IntoSql};
#[cfg(feature = "postgres")]
use tokio_postgres::{self, types::ToSql};

// TODO: cfg feature for this re-exports, as date-time or something
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};

/// Defines a trait for represent type bounds against the allowed
/// data types supported by Canyon to be used as query parameters.
pub trait QueryParameter: Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;

    #[cfg(feature = "postgres")]
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync);
    #[cfg(feature = "mssql")]
    fn as_sqlserver_param(&self) -> ColumnData<'_>;
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> Value;
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
        ColumnData::I16(self.copied())
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
        // SQL Server has no unsigned integer type. BIGINT represents every u32 losslessly.
        ColumnData::I64(Some(i64::from(*self)))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
        ColumnData::I64(self.map(i64::from))
    }
    #[cfg(feature = "mysql")]
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.to_value()
    }
}

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
    fn as_mysql_param(&self) -> Value {
        self.naive_utc().to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.as_ref().map(DateTime::naive_utc).to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.naive_utc().to_value()
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
    fn as_mysql_param(&self) -> Value {
        self.as_ref().map(DateTime::naive_utc).to_value()
    }
}

#[cfg(all(test, feature = "mssql"))]
mod mssql_tests {
    use super::QueryParameter;
    use tiberius::ColumnData;

    static VALUE: i16 = 42;

    #[test]
    fn optional_i16_parameter_preserves_some_and_none() {
        let some: Option<&'static i16> = Some(&VALUE);
        let none: Option<&'static i16> = None;

        assert!(matches!(
            some.as_sqlserver_param(),
            ColumnData::I16(Some(42))
        ));
        assert!(matches!(none.as_sqlserver_param(), ColumnData::I16(None)));
    }

    #[test]
    fn u32_parameter_uses_lossless_sql_server_bigint() {
        let first_value_beyond_i32 = i32::MAX as u32 + 1;

        assert!(matches!(
            0_u32.as_sqlserver_param(),
            ColumnData::I64(Some(0))
        ));
        assert!(matches!(
            (i32::MAX as u32).as_sqlserver_param(),
            ColumnData::I64(Some(value)) if value == i64::from(i32::MAX)
        ));
        assert!(matches!(
            first_value_beyond_i32.as_sqlserver_param(),
            ColumnData::I64(Some(value)) if value == i64::from(first_value_beyond_i32)
        ));
        assert!(matches!(
            u32::MAX.as_sqlserver_param(),
            ColumnData::I64(Some(value)) if value == i64::from(u32::MAX)
        ));
    }

    #[test]
    fn optional_u32_parameter_preserves_some_and_none() {
        assert!(matches!(
            Some(u32::MAX).as_sqlserver_param(),
            ColumnData::I64(Some(value)) if value == i64::from(u32::MAX)
        ));
        assert!(matches!(
            Option::<u32>::None.as_sqlserver_param(),
            ColumnData::I64(None)
        ));
    }
}

#[cfg(all(test, feature = "postgres"))]
mod postgres_tests {
    use super::QueryParameter;
    use tokio_postgres::types::{IsNull, Type, private::BytesMut};

    static VALUE: i16 = 42;

    #[test]
    fn optional_i16_parameter_preserves_some_and_none() {
        let some: Option<&'static i16> = Some(&VALUE);
        let none: Option<&'static i16> = None;
        let mut some_bytes = BytesMut::new();
        let mut none_bytes = BytesMut::new();

        let some_nullability = some
            .as_postgres_param()
            .to_sql_checked(&Type::INT2, &mut some_bytes)
            .unwrap();
        let none_nullability = none
            .as_postgres_param()
            .to_sql_checked(&Type::INT2, &mut none_bytes)
            .unwrap();

        assert!(matches!(some_nullability, IsNull::No));
        assert_eq!(some_bytes.as_ref(), VALUE.to_be_bytes());
        assert!(matches!(none_nullability, IsNull::Yes));
        assert!(none_bytes.is_empty());
    }
}

#[cfg(all(test, feature = "mysql"))]
mod mysql_tests {
    use super::QueryParameter;
    use chrono::{DateTime, FixedOffset, TimeZone, Utc};
    use mysql_async::Value;

    static I16_VALUE: i16 = 42;

    #[test]
    fn optional_i16_parameter_preserves_some_and_none() {
        let some: Option<&'static i16> = Some(&I16_VALUE);
        let none: Option<&'static i16> = None;

        assert_eq!(some.as_mysql_param(), Value::Int(42));
        assert_eq!(none.as_mysql_param(), Value::NULL);
    }

    #[test]
    fn mysql_datetime_parameters_are_normalized_to_utc() {
        let fixed_offset = FixedOffset::east_opt(2 * 60 * 60).unwrap();
        let fixed_datetime = fixed_offset
            .with_ymd_and_hms(2024, 1, 2, 3, 4, 5)
            .single()
            .unwrap();
        let utc_datetime = Utc.with_ymd_and_hms(2024, 1, 2, 1, 4, 5).unwrap();
        let expected = Value::Date(2024, 1, 2, 1, 4, 5, 0);

        assert_eq!(fixed_datetime.as_mysql_param(), expected);
        assert_eq!(utc_datetime.as_mysql_param(), expected);
        assert_eq!(Some(fixed_datetime).as_mysql_param(), expected);
        assert_eq!(Some(utc_datetime).as_mysql_param(), expected);

        let no_fixed_datetime: Option<DateTime<FixedOffset>> = None;
        let no_utc_datetime: Option<DateTime<Utc>> = None;
        assert_eq!(no_fixed_datetime.as_mysql_param(), Value::NULL);
        assert_eq!(no_utc_datetime.as_mysql_param(), Value::NULL);
    }
}
