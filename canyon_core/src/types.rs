//! Backend-neutral column types and the Rust types that map to them.
//!
//! [`SqlType`] says what a column must hold; each backend picks its own native
//! type for it. [`ColumnType`] maps a Rust type to its [`SqlType`]. Canyon only
//! implements it for types whose values round-trip on every supported database.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

/// What a column must hold, independent of the database.
///
/// The migrations snapshot stores it as JSON (`{"type":"text","max_len":100}`),
/// so variant and field names are part of the stored format.
///
/// ```
/// use canyon_core::types::SqlType;
///
/// let name = SqlType::Text { max_len: None }.with_length(100);
/// assert_eq!(name, SqlType::Text { max_len: Some(100) });
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum SqlType {
    /// `bool`.
    Bool,
    /// 16-bit integer, `i16`.
    SmallInt,
    /// 32-bit integer, `i32`.
    Int,
    /// 64-bit integer, `i64`.
    BigInt,
    /// `String`.
    Text {
        /// Maximum length, `None` for unlimited text. See [`SqlType::with_length`].
        max_len: Option<u32>,
    },
    /// `NaiveDate`.
    Date,
    /// `NaiveTime`.
    Time {
        /// Fractional-second digits.
        precision: u8,
    },
    /// `NaiveDateTime`, without time zone.
    Timestamp {
        /// Fractional-second digits.
        precision: u8,
    },
}

impl SqlType {
    /// Microseconds: the most PostgreSQL and MySQL can store.
    pub const DEFAULT_PRECISION: u8 = 6;

    /// `SmallInt`, `Int` or `BigInt`.
    pub const fn is_integer(self) -> bool {
        matches!(self, Self::SmallInt | Self::Int | Self::BigInt)
    }

    /// Text with a maximum length of `n`. PostgreSQL and MySQL count characters;
    /// SQL Server counts UTF-16 code units, so a character outside the BMP (an emoji)
    /// counts twice there.
    ///
    /// # Panics
    /// If `self` is not text or `n` is 0. In const context (generated code) this is a
    /// compile error.
    pub const fn with_length(self, n: u32) -> Self {
        assert!(n > 0, "`length` must be at least 1");
        match self {
            Self::Text { .. } => Self::Text { max_len: Some(n) },
            _ => panic!("`length` only applies to text fields"),
        }
    }

    /// Time or timestamp with `p` fractional-second digits.
    ///
    /// # Panics
    /// If `self` is not a time or timestamp.
    pub const fn with_precision(self, p: u8) -> Self {
        match self {
            Self::Time { .. } => Self::Time { precision: p },
            Self::Timestamp { .. } => Self::Timestamp { precision: p },
            _ => panic!("`precision` only applies to time and timestamp fields"),
        }
    }
}

/// Maps a Rust type to the [`SqlType`] of its column.
///
/// Not to be confused with [`crate::column::NativeColumnType`], the driver's type of a
/// column in a result row.
///
/// ```
/// use canyon_core::types::{ColumnType, SqlType};
///
/// assert_eq!(<i32 as ColumnType>::SQL, SqlType::Int);
/// assert!(<Option<i32> as ColumnType>::NULLABLE);
/// ```
///
/// A nested `Option` is a compile error, no database tells `Some(None)` from `None`:
///
/// ```compile_fail,E0080
/// use canyon_core::types::ColumnType;
///
/// let _ = <Option<Option<i32>> as ColumnType>::SQL;
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be the type of a Canyon entity field",
    label = "no column type for `{Self}`",
    note = "supported: bool, i16, i32, i64, String, NaiveDate, NaiveTime, NaiveDateTime, and Option of them",
    note = "for u8 / u16 / u32 use the next wider signed type; u64 has no column type"
)]
pub trait ColumnType {
    /// The column type for `Self`.
    const SQL: SqlType;
    /// `true` only for `Option<T>`: the column accepts NULL.
    const NULLABLE: bool = false;
}

impl<T: ColumnType> ColumnType for Option<T> {
    // No database tells `Some(None)` from `None`
    const SQL: SqlType = {
        assert!(!T::NULLABLE, "a field can't be `Option<Option<_>>`");
        T::SQL
    };
    const NULLABLE: bool = true;
}

// A new impl needs its round-trip test in tests/types/mod.rs and its name in the note above
impl ColumnType for bool {
    const SQL: SqlType = SqlType::Bool;
}
impl ColumnType for i16 {
    const SQL: SqlType = SqlType::SmallInt;
}
impl ColumnType for i32 {
    const SQL: SqlType = SqlType::Int;
}
impl ColumnType for i64 {
    const SQL: SqlType = SqlType::BigInt;
}
impl ColumnType for String {
    const SQL: SqlType = SqlType::Text { max_len: None };
}
impl ColumnType for NaiveDate {
    const SQL: SqlType = SqlType::Date;
}
impl ColumnType for NaiveTime {
    const SQL: SqlType = SqlType::Time {
        precision: SqlType::DEFAULT_PRECISION,
    };
}
impl ColumnType for NaiveDateTime {
    const SQL: SqlType = SqlType::Timestamp {
        precision: SqlType::DEFAULT_PRECISION,
    };
}

#[cfg(test)]
mod tests {
    use super::{ColumnType, SqlType};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    #[test]
    fn every_supported_type_maps_to_its_sql_type() {
        assert_eq!(<bool as ColumnType>::SQL, SqlType::Bool);
        assert_eq!(<i16 as ColumnType>::SQL, SqlType::SmallInt);
        assert_eq!(<i32 as ColumnType>::SQL, SqlType::Int);
        assert_eq!(<i64 as ColumnType>::SQL, SqlType::BigInt);
        assert_eq!(<String as ColumnType>::SQL, SqlType::Text { max_len: None });
        assert_eq!(<NaiveDate as ColumnType>::SQL, SqlType::Date);
        assert_eq!(
            <NaiveTime as ColumnType>::SQL,
            SqlType::Time { precision: 6 }
        );
        assert_eq!(
            <NaiveDateTime as ColumnType>::SQL,
            SqlType::Timestamp { precision: 6 }
        );
    }

    #[test]
    fn option_keeps_the_sql_type() {
        assert_eq!(<Option<i32> as ColumnType>::SQL, SqlType::Int);
    }

    // Only `Option` is nullable, checked at compile time
    const _: () = assert!(<Option<i32> as ColumnType>::NULLABLE);
    const _: () = assert!(!<i32 as ColumnType>::NULLABLE);
    const _: () = assert!(<Option<String> as ColumnType>::NULLABLE);

    #[test]
    fn with_length_sets_the_text_length() {
        assert_eq!(
            <String as ColumnType>::SQL.with_length(100),
            SqlType::Text { max_len: Some(100) }
        );
    }

    #[test]
    #[should_panic(expected = "`length` only applies to text fields")]
    fn with_length_rejects_non_text() {
        let _ = SqlType::Int.with_length(10);
    }

    #[test]
    #[should_panic(expected = "`length` must be at least 1")]
    fn with_length_rejects_zero() {
        let _ = <String as ColumnType>::SQL.with_length(0);
    }

    #[test]
    fn with_precision_sets_time_and_timestamp_precision() {
        assert_eq!(
            <NaiveTime as ColumnType>::SQL.with_precision(3),
            SqlType::Time { precision: 3 }
        );
        assert_eq!(
            <NaiveDateTime as ColumnType>::SQL.with_precision(0),
            SqlType::Timestamp { precision: 0 }
        );
    }

    #[test]
    #[should_panic(expected = "`precision` only applies to time and timestamp fields")]
    fn with_precision_rejects_other_types() {
        let _ = SqlType::Date.with_precision(3);
    }

    #[test]
    fn is_integer_only_for_integers() {
        assert!(SqlType::SmallInt.is_integer() && SqlType::BigInt.is_integer());
        assert!(!SqlType::Bool.is_integer());
    }

    // Usable in const context: generated code asserts with it
    const _: () = assert!(<i32 as ColumnType>::SQL.is_integer());

    // The stored format: changing any of these breaks reading old snapshots
    #[test]
    fn sql_type_json_is_stable() {
        let cases = [
            (SqlType::Bool, r#"{"type":"bool"}"#),
            (SqlType::SmallInt, r#"{"type":"small_int"}"#),
            (SqlType::Int, r#"{"type":"int"}"#),
            (SqlType::BigInt, r#"{"type":"big_int"}"#),
            (
                SqlType::Text { max_len: None },
                r#"{"type":"text","max_len":null}"#,
            ),
            (
                SqlType::Text { max_len: Some(100) },
                r#"{"type":"text","max_len":100}"#,
            ),
            (SqlType::Date, r#"{"type":"date"}"#),
            (
                SqlType::Time { precision: 6 },
                r#"{"type":"time","precision":6}"#,
            ),
            (
                SqlType::Timestamp { precision: 3 },
                r#"{"type":"timestamp","precision":3}"#,
            ),
        ];
        for (ty, json) in cases {
            assert_eq!(serde_json::to_string(&ty).unwrap(), json);
            assert_eq!(serde_json::from_str::<SqlType>(json).unwrap(), ty);
        }
    }
}
