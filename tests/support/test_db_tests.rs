use super::test_db::{
    MAX_SCHEMA_SUFFIX_LEN, PREFIX, STALE_AFTER_SECS, TestDb, create_database, created_at, db_name,
    drop_database, is_safe_name, is_stale, is_valid_schema_suffix, list_test_databases, now_secs,
    random_suffix, sweep_stale,
};
use canyon_sql::connection::{DatabaseType, DbConnection};
use canyon_sql::core::Canyon;
use canyon_sql::runtime::futures::FutureExt;
use canyon_sql::runtime::futures::future::join_all;
use canyon_sql::{CanyonError, CanyonResult};
use std::panic::AssertUnwindSafe;

#[test]
fn db_name_has_prefix_timestamp_suffix_and_tag() {
    let name = db_name("types", 1_760_000_000, 0xab);
    assert_eq!(name, "canyon_test_1760000000_000000ab_types");
}

#[test]
fn db_name_sanitizes_and_truncates_the_tag() {
    let name = db_name(
        "Weird-Tag With Spaces & a very long name 1234567890",
        1_760_000_000,
        1,
    );
    assert!(name.starts_with(PREFIX));
    assert!(name.len() <= 63, "{name} is {} bytes", name.len());
    assert!(is_safe_name(&name), "{name}");
}

#[test]
fn longest_db_name_leaves_room_for_a_schema_suffix() {
    let name = db_name(&"t".repeat(100), u64::from(u32::MAX), u32::MAX);
    let companion = format!("{name}_{}", "s".repeat(MAX_SCHEMA_SUFFIX_LEN));
    assert!(is_safe_name(&companion), "{companion}");
}

#[test]
fn schema_suffixes_are_short_lowercase_names() {
    assert!(is_valid_schema_suffix("s1"));
    assert!(is_valid_schema_suffix(&"s".repeat(MAX_SCHEMA_SUFFIX_LEN)));
    assert!(!is_valid_schema_suffix(""));
    assert!(!is_valid_schema_suffix("S1"));
    assert!(!is_valid_schema_suffix("s-1"));
    assert!(!is_valid_schema_suffix(
        &"s".repeat(MAX_SCHEMA_SUFFIX_LEN + 1)
    ));
}

#[test]
fn created_at_reads_back_the_timestamp() {
    let name = db_name("x", 1_760_000_000, 7);
    assert_eq!(created_at(&name), Some(1_760_000_000));
}

#[test]
fn created_at_reads_back_every_db_name() {
    for (tag, t) in [
        ("", 1_760_000_000),
        ("x", 0),
        ("Ä-ü", 42),
        ("t", 9_999_999_999),
    ] {
        assert_eq!(created_at(&db_name(tag, t, 7)), Some(t), "{tag:?} {t}");
    }
}

#[test]
fn created_at_ignores_names_that_are_not_ours() {
    assert_eq!(created_at("postgres"), None);
    assert_eq!(created_at("canyon_test_notanumber_1_x"), None);
    assert_eq!(created_at("canyon_testing_1760000000"), None);
    assert_eq!(created_at("canyon_test_2024_backup"), None);
    assert_eq!(created_at("canyon_test_1760000000_000000ab"), None);
    assert_eq!(created_at("canyon_test_1760000000_nothex00_x"), None);
}

#[test]
fn is_stale_starts_after_the_threshold() {
    let name = db_name("x", 1_760_000_000, 7);
    let limit = 1_760_000_000 + STALE_AFTER_SECS;
    assert!(!is_stale(&name, limit));
    assert!(is_stale(&name, limit + 1));
    assert!(!is_stale("canyon_test_2024_backup", limit + 1));
}

#[test]
fn is_safe_name_rejects_quotes_and_foreign_names() {
    assert!(!is_safe_name("canyon_test_1_\"; DROP DATABASE x; --"));
    assert!(!is_safe_name("canyon_test_1_]"));
    assert!(!is_safe_name("other_db"));
    assert!(!is_safe_name(&format!("{PREFIX}{}", "a".repeat(64))));
}

#[test]
fn random_suffix_differs_between_calls() {
    let a = random_suffix();
    let b = random_suffix();
    assert_ne!(a, b);
}

async fn create_use_and_drop(datasource: &str) -> CanyonResult<()> {
    // `new` + `cleanup` on purpose: this test checks `cleanup` itself
    let db = TestDb::new(datasource, "harness_create").await?;
    let name = db.name().to_owned();

    let used = async {
        db.conn()
            .execute("CREATE TABLE harness_probe (c INT)", &[])
            .await?;
        let inserted = db
            .conn()
            .execute("INSERT INTO harness_probe (c) VALUES (1)", &[])
            .await?;
        let listed = list_test_databases(datasource).await?.contains(&name);
        Ok::<_, CanyonError>((inserted, listed))
    }
    .await;
    db.cleanup().await?;
    let (inserted, listed) = used?;
    assert_eq!(inserted, 1);
    assert!(listed, "{name} should be listed");
    assert!(!list_test_databases(datasource).await?.contains(&name));
    Ok(())
}
per_backend!(harness_create_use_and_drop, super::create_use_and_drop);

async fn test_db_is_isolated_from_the_datasource_db(datasource: &str) -> CanyonResult<()> {
    let seen_from_datasource_db = TestDb::with(datasource, "harness_isolation", async |db| {
        db.conn()
            .execute("CREATE TABLE harness_isolation_probe (c INT)", &[])
            .await?;
        let admin = Canyon::instance()?.get_connection(datasource)?;
        let lookup = match admin.get_db_type() {
            #[cfg(feature = "postgres")]
            DatabaseType::PostgreSql => {
                "SELECT table_name FROM information_schema.tables WHERE table_name = 'harness_isolation_probe'"
            }
            #[cfg(feature = "mssql")]
            DatabaseType::SqlServer => {
                "SELECT table_name FROM information_schema.tables WHERE table_name = 'harness_isolation_probe'"
            }
            // MySQL's information_schema covers every database of the server
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => {
                "SELECT table_name FROM information_schema.tables \
                 WHERE table_name = 'harness_isolation_probe' AND table_schema = DATABASE()"
            }
        };
        admin.query_rows(lookup, &[]).await
    })
    .await?;
    assert!(seen_from_datasource_db.is_empty());
    Ok(())
}
per_backend!(
    harness_isolation,
    super::test_db_is_isolated_from_the_datasource_db
);

// Not through `TestDb`, whose `new` sweeps too and would hide a broken `sweep_stale`
async fn sweep_drops_only_stale_databases(datasource: &str) -> CanyonResult<()> {
    let old = now_secs() - STALE_AFTER_SECS - 60;
    let stale = db_name("harness_stale", old, random_suffix());
    let fresh = db_name("harness_fresh", now_secs(), random_suffix());
    create_database(datasource, &stale).await?;
    create_database(datasource, &fresh).await?;

    let swept = sweep_stale(datasource).await;
    let left = list_test_databases(datasource).await?;
    drop_database(datasource, &fresh).await?;

    swept?;
    assert!(!left.contains(&stale), "{stale} should have been swept");
    assert!(left.contains(&fresh), "a fresh database must not be swept");
    Ok(())
}
per_backend!(harness_sweep, super::sweep_drops_only_stale_databases);

async fn companion_schema_is_cleaned_up(datasource: &str) -> CanyonResult<()> {
    let db = TestDb::new(datasource, "harness_schema").await?;
    let s = db.schema("s1");
    let create = match db.db_type() {
        #[cfg(feature = "postgres")]
        DatabaseType::PostgreSql => format!("CREATE SCHEMA \"{s}\""),
        #[cfg(feature = "mssql")]
        // CREATE SCHEMA must be alone in its batch: through RPC it fails, through EXEC it works
        DatabaseType::SqlServer => format!("EXEC(N'CREATE SCHEMA [{s}]')"),
        #[cfg(feature = "mysql")]
        DatabaseType::MySQL => format!("CREATE DATABASE `{s}`"),
    };
    let created = db.conn().execute(&create, &[]).await;
    let name = db.name().to_owned();
    db.cleanup().await?;
    created?;
    let left = list_test_databases(datasource).await?;
    assert!(left.iter().all(|n| !n.starts_with(&name)), "{left:?}");
    Ok(())
}
per_backend!(
    harness_companion_schema,
    super::companion_schema_is_cleaned_up
);

async fn with_drops_the_database_when_the_body_fails(datasource: &str) -> CanyonResult<()> {
    let mut name = String::new();
    let result = TestDb::with(datasource, "harness_with_err", async |db| {
        name = db.name().to_owned();
        db.conn().execute("SELECT * FROM no_such_table", &[]).await
    })
    .await;
    assert!(result.is_err(), "the body's error must come back");
    assert!(!name.is_empty(), "the body never ran: {:?}", result.err());
    assert!(!list_test_databases(datasource).await?.contains(&name));
    Ok(())
}
per_backend!(
    harness_with_body_error,
    super::with_drops_the_database_when_the_body_fails
);

async fn with_drops_the_database_when_the_body_panics(datasource: &str) -> CanyonResult<()> {
    let mut name = String::new();
    let outcome = AssertUnwindSafe(TestDb::with(datasource, "harness_with_panic", async |db| {
        name = db.name().to_owned();
        panic!("test body panicked on purpose");
        #[allow(unreachable_code)]
        Ok(())
    }))
    .catch_unwind()
    .await;
    if let Ok(result) = outcome {
        result?;
        panic!("the body's panic must come back");
    }
    assert!(!name.is_empty());
    assert!(!list_test_databases(datasource).await?.contains(&name));
    Ok(())
}
per_backend!(
    harness_with_body_panic,
    super::with_drops_the_database_when_the_body_panics
);

// CI runs one test at a time, so the SQL Server drop lock only gets raced here
async fn concurrent_drops_of_one_database_all_succeed(datasource: &str) -> CanyonResult<()> {
    let name = db_name("harness_race", now_secs(), random_suffix());
    create_database(datasource, &name).await?;
    // Four at once: with two, a missing lock only fails some runs
    let drops = join_all((0..4).map(|_| drop_database(datasource, &name))).await;
    drops.into_iter().collect::<CanyonResult<Vec<()>>>()?;
    assert!(!list_test_databases(datasource).await?.contains(&name));
    Ok(())
}
per_backend!(
    harness_concurrent_drop,
    super::concurrent_drops_of_one_database_all_succeed
);
