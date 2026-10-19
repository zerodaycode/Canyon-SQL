//! Every `ColumnType` round-trips through its canonical column type on every backend.
//! Values are microsecond-exact on purpose: drivers truncate or round anything finer.

use crate::support::per_backend;
use crate::support::test_db::TestDb;
use canyon_sql::CanyonResult;
use canyon_sql::connection::{DatabaseType, DbConnection};
use canyon_sql::core::SqlType;
use canyon_sql::date_time::{NaiveDate, NaiveDateTime, NaiveTime};
use canyon_sql::macros::CanyonMapper;

/// The native column type each backend uses for a `SqlType`.
pub fn canonical(db: DatabaseType, ty: SqlType) -> String {
    use SqlType::*;
    match (db, ty) {
        #[cfg(feature = "postgres")]
        (DatabaseType::PostgreSql, t) => match t {
            Bool => "boolean".into(),
            SmallInt => "smallint".into(),
            Int => "integer".into(),
            BigInt => "bigint".into(),
            Text { max_len: None } => "text".into(),
            Text { max_len: Some(n) } => format!("varchar({n})"),
            Date => "date".into(),
            Time { precision } => format!("time({precision})"),
            Timestamp { precision } => format!("timestamp({precision})"),
            _ => unreachable!("no canonical type for {t:?}"),
        },
        #[cfg(feature = "mssql")]
        (DatabaseType::SqlServer, t) => match t {
            Bool => "bit".into(),
            SmallInt => "smallint".into(),
            Int => "int".into(),
            BigInt => "bigint".into(),
            Text { max_len: None } => "nvarchar(max)".into(),
            Text { max_len: Some(n) } => format!("nvarchar({n})"),
            Date => "date".into(),
            Time { precision } => format!("time({precision})"),
            Timestamp { precision } => format!("datetime2({precision})"),
            _ => unreachable!("no canonical type for {t:?}"),
        },
        #[cfg(feature = "mysql")]
        (DatabaseType::MySQL, t) => match t {
            Bool => "tinyint(1)".into(),
            SmallInt => "smallint".into(),
            Int => "int".into(),
            BigInt => "bigint".into(),
            Text { max_len: None } => "longtext".into(),
            Text { max_len: Some(n) } => format!("varchar({n})"),
            Date => "date".into(),
            Time { precision } => format!("time({precision})"),
            Timestamp { precision } => format!("datetime({precision})"),
            _ => unreachable!("no canonical type for {t:?}"),
        },
    }
}

/// `INSERT INTO <table> (c) VALUES (<first placeholder>)`.
fn insert_into(db: &TestDb, table: &str) -> String {
    let placeholder = match db.db_type() {
        #[cfg(feature = "postgres")]
        DatabaseType::PostgreSql => "$1",
        #[cfg(feature = "mssql")]
        DatabaseType::SqlServer => "@P1",
        #[cfg(feature = "mysql")]
        DatabaseType::MySQL => "?",
    };
    format!("INSERT INTO {table} (c) VALUES ({placeholder})")
}

/// Creates `rt` with one `$native` column. For each value: writes it, reads it back
/// as `$ty` and compares. Then drops `rt`.
macro_rules! round_trips {
    ($db:expr, $native:expr, $ty:ty, [$($value:expr),+ $(,)?]) => {
        round_trips!($db, $native, $ty, [$($value),+].into_iter())
    };
    ($db:expr, $native:expr, $ty:ty, $values:expr) => {{
        let db: &TestDb = $db;
        let native = $native;
        db.conn()
            .execute(&format!("CREATE TABLE rt (c {native})"), &[])
            .await?;
        for value in $values {
            let value: $ty = value;
            db.conn().execute("DELETE FROM rt", &[]).await?;
            db.conn().execute(&insert_into(db, "rt"), &[&value]).await?;
            let back = db
                .conn()
                .query_one_for::<$ty>("SELECT c FROM rt", &[])
                .await?;
            assert_eq!(back, value, "{} through {native}", stringify!($ty));
        }
        db.conn().execute("DROP TABLE rt", &[]).await?;
    }};
}

/// One row type per `Option<T>`, read through `CanyonMapper` like entity fields are.
/// Each lives in its own module (the derive adds `use`s at module level), and `$ty` is
/// an `ident` so the derive sees the type's own tokens (it special-cases `String`).
macro_rules! option_rows {
    ($($module:ident: $ty:ident),+ $(,)?) => {$(
        mod $module {
            use super::*;

            #[derive(CanyonMapper, Debug, PartialEq)]
            pub struct Row {
                pub c: Option<$ty>,
            }
        }
    )+};
}
option_rows!(
    opt_bool: bool,
    opt_i16: i16,
    opt_i32: i32,
    opt_i64: i64,
    opt_string: String,
    opt_date: NaiveDate,
    opt_time: NaiveTime,
    opt_timestamp: NaiveDateTime,
);

/// Creates `rt` with one nullable `$native` column, writes `None` and `Some($value)`,
/// and reads each back through `$module::Row`. Then drops `rt`.
macro_rules! option_round_trips {
    ($db:expr, $native:expr, $module:ident, $value:expr) => {{
        let db: &TestDb = $db;
        let native = $native;
        db.conn()
            .execute(&format!("CREATE TABLE rt (c {native} NULL)"), &[])
            .await?;
        for value in [None, Some($value)] {
            db.conn().execute("DELETE FROM rt", &[]).await?;
            db.conn().execute(&insert_into(db, "rt"), &[&value]).await?;
            let back = db
                .conn()
                .query_one::<$module::Row>("SELECT c FROM rt", &[])
                .await?;
            assert_eq!(
                back,
                Some($module::Row { c: value.clone() }),
                "{value:?} through {native}"
            );
        }
        db.conn().execute("DROP TABLE rt", &[]).await?;
    }};
}

fn time(h: u32, m: u32, s: u32, micro: u32) -> NaiveTime {
    NaiveTime::from_hms_micro_opt(h, m, s, micro).expect("valid time")
}
fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}
fn timestamp(d: NaiveDate, t: NaiveTime) -> NaiveDateTime {
    NaiveDateTime::new(d, t)
}

async fn scalars_round_trip(datasource: &str) -> CanyonResult<()> {
    TestDb::with(datasource, "types_scalars", async |db| {
        let c = |ty| canonical(db.db_type(), ty);
        round_trips!(db, c(SqlType::Bool), bool, [false, true]);
        round_trips!(db, c(SqlType::SmallInt), i16, [i16::MIN, -1, 0, i16::MAX]);
        round_trips!(db, c(SqlType::Int), i32, [i32::MIN, 0, i32::MAX]);
        round_trips!(db, c(SqlType::BigInt), i64, [i64::MIN, 0, i64::MAX]);
        Ok(())
    })
    .await
}
per_backend!(types_scalars_round_trip, super::scalars_round_trip);

fn text_samples() -> Vec<String> {
    [
        "",
        "plain ascii",
        "ñandú 中文 😀",
        "quote ' and \" and ] and ` and --",
    ]
    .map(str::to_owned)
    .to_vec()
}

async fn text_round_trip(datasource: &str) -> CanyonResult<()> {
    TestDb::with(datasource, "types_text", async |db| {
        let unlimited = canonical(db.db_type(), SqlType::Text { max_len: None });
        let hundred = canonical(db.db_type(), SqlType::Text { max_len: Some(100) });
        round_trips!(db, &unlimited, String, text_samples());
        round_trips!(db, &unlimited, String, ["x".repeat(10_000)]);
        round_trips!(db, &hundred, String, text_samples());
        // 50 emoji fit everywhere: SQL Server counts them as 100 UTF-16 code units
        round_trips!(db, &hundred, String, ["é".repeat(100), "😀".repeat(50)]);
        Ok(())
    })
    .await
}
per_backend!(types_text_round_trip, super::text_round_trip);

/// `length` counts characters on PostgreSQL and MySQL but UTF-16 code units on SQL
/// Server, so 100 emoji only fit in a length-100 column outside SQL Server.
async fn text_length_units(datasource: &str) -> CanyonResult<()> {
    TestDb::with(datasource, "types_text_len", async |db| {
        let hundred = canonical(db.db_type(), SqlType::Text { max_len: Some(100) });
        let emoji = "😀".repeat(100);
        db.conn()
            .execute(&format!("CREATE TABLE rt (c {hundred})"), &[])
            .await?;
        // Rejected, or cut short on the way in
        let stored_whole = match db.conn().execute(&insert_into(db, "rt"), &[&emoji]).await {
            Ok(_) => {
                db.conn()
                    .query_one_for::<String>("SELECT c FROM rt", &[])
                    .await?
                    == emoji
            }
            Err(_) => false,
        };
        #[cfg(feature = "mssql")]
        let counts_code_units = matches!(db.db_type(), DatabaseType::SqlServer);
        #[cfg(not(feature = "mssql"))]
        let counts_code_units = false;
        assert_eq!(
            stored_whole, !counts_code_units,
            "100 emoji through {hundred}"
        );
        Ok(())
    })
    .await
}
per_backend!(types_text_length_units, super::text_length_units);

async fn temporal_round_trip(datasource: &str) -> CanyonResult<()> {
    TestDb::with(datasource, "types_temporal", async |db| {
        let k = db.db_type();
        let p = SqlType::DEFAULT_PRECISION;
        // MySQL's supported DATE / DATETIME range starts at 1000-01-01
        round_trips!(
            db,
            canonical(k, SqlType::Date),
            NaiveDate,
            [
                date(1000, 1, 1),
                date(1970, 1, 1),
                date(2026, 10, 19),
                date(9999, 12, 31),
            ]
        );
        round_trips!(
            db,
            canonical(k, SqlType::Time { precision: p }),
            NaiveTime,
            [
                time(0, 0, 0, 0),
                time(12, 34, 56, 123_456),
                time(23, 59, 59, 999_999),
            ]
        );
        round_trips!(
            db,
            canonical(k, SqlType::Timestamp { precision: p }),
            NaiveDateTime,
            [
                timestamp(date(1000, 1, 1), time(0, 0, 0, 0)),
                timestamp(date(2026, 10, 19), time(12, 34, 56, 123_456)),
                timestamp(date(9999, 12, 31), time(23, 59, 59, 999_999)),
            ]
        );
        Ok(())
    })
    .await
}
per_backend!(types_temporal_round_trip, super::temporal_round_trip);

async fn options_round_trip(datasource: &str) -> CanyonResult<()> {
    TestDb::with(datasource, "types_options", async |db| {
        let c = |ty| canonical(db.db_type(), ty);
        let p = SqlType::DEFAULT_PRECISION;
        option_round_trips!(db, c(SqlType::Bool), opt_bool, true);
        option_round_trips!(db, c(SqlType::SmallInt), opt_i16, -7_i16);
        option_round_trips!(db, c(SqlType::Int), opt_i32, 7_i32);
        option_round_trips!(db, c(SqlType::BigInt), opt_i64, 7_i64);
        option_round_trips!(
            db,
            c(SqlType::Text { max_len: None }),
            opt_string,
            "x".to_owned()
        );
        option_round_trips!(db, c(SqlType::Date), opt_date, date(2026, 10, 19));
        option_round_trips!(
            db,
            c(SqlType::Time { precision: p }),
            opt_time,
            time(1, 2, 3, 4)
        );
        option_round_trips!(
            db,
            c(SqlType::Timestamp { precision: p }),
            opt_timestamp,
            timestamp(date(2026, 10, 19), time(1, 2, 3, 4))
        );
        Ok(())
    })
    .await
}
per_backend!(types_options_round_trip, super::options_round_trip);

/// Writes `$value` into a `$native` column, reads it back as `$ty` and checks that the
/// backend's driver is strict where it should be (`check_strict`).
macro_rules! cross_width {
    ($db:expr, $native:literal, $ty:ty, $value:expr) => {{
        let db: &TestDb = $db;
        let value: $ty = $value;
        db.conn()
            .execute(concat!("CREATE TABLE w (c ", $native, ")"), &[])
            .await?;
        let write = db.conn().execute(&insert_into(db, "w"), &[&value]).await;
        if write.is_err() {
            // Fallback row so the read below still has something to read
            db.conn()
                .execute("INSERT INTO w (c) VALUES (7)", &[])
                .await?;
        }
        let read = db.conn().query_one_for::<$ty>("SELECT c FROM w", &[]).await;
        db.conn().execute("DROP TABLE w", &[]).await?;
        check_strict(
            db.db_type(),
            &write,
            &read,
            concat!(stringify!($ty), " on ", $native),
        );
    }};
}

#[cfg(feature = "postgres")]
fn is_wrong_type<T>(result: &CanyonResult<T>) -> bool {
    use canyon_sql::core::{CanyonError, QueryError};
    use canyon_sql::db_clients::tokio_postgres::types::WrongType;
    matches!(result, Err(CanyonError::Query(QueryError::Postgres(e)))
        if std::error::Error::source(&**e).is_some_and(|s| s.is::<WrongType>()))
}

#[cfg(feature = "mssql")]
fn is_conversion_error<T>(result: &CanyonResult<T>) -> bool {
    use canyon_sql::core::{CanyonError, QueryError};
    use canyon_sql::db_clients::tiberius::error::Error;
    matches!(result, Err(CanyonError::Query(QueryError::SqlServer(e)))
        if matches!(**e, Error::Conversion(_)))
}

// Each backend checks the side where its driver is strict
#[allow(unused_variables)]
fn check_strict<T: std::fmt::Debug>(
    db_type: DatabaseType,
    write: &CanyonResult<u64>,
    read: &CanyonResult<T>,
    what: &str,
) {
    match db_type {
        #[cfg(feature = "postgres")]
        DatabaseType::PostgreSql => assert!(
            is_wrong_type(write),
            "PostgreSQL must reject {what} on write: {write:?}"
        ),
        #[cfg(feature = "mssql")]
        DatabaseType::SqlServer => assert!(
            is_conversion_error(read),
            "SQL Server must refuse to read {what}: {read:?}"
        ),
        #[cfg(feature = "mysql")]
        DatabaseType::MySQL => assert!(
            write.is_ok() && read.is_ok(),
            "MySQL converts {what}: {write:?} {read:?}"
        ),
    }
}

/// Integers only match their exact width: a narrower field on a wider column fails on
/// PostgreSQL (write) and SQL Server (read). MySQL converts.
async fn integer_widths_are_strict(datasource: &str) -> CanyonResult<()> {
    TestDb::with(datasource, "types_widths", async |db| {
        cross_width!(db, "INT", i16, 7);
        cross_width!(db, "BIGINT", i32, 7);
        Ok(())
    })
    .await
}
per_backend!(
    types_integer_widths_are_strict,
    super::integer_widths_are_strict
);
