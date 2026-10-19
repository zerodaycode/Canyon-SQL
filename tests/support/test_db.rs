//! Throwaway databases for tests that change schemas.
//!
//! A [`TestDb`] lives on the server behind one of the datasources of
//! `tests/canyon.toml`. Use [`TestDb::with`]: it drops the database at the end,
//! also when the test fails (not when the test is cancelled, see `with`). Names are `canyon_test_<unix-secs>_<random>_<tag>`,
//! so [`sweep_stale`] can drop the ones a crashed run left behind.

use canyon_sql::connection::{DatabaseConnector, DatabaseType, DbConnection};
use canyon_sql::core::{Canyon, CanyonRows};
use canyon_sql::runtime::futures::FutureExt;
use canyon_sql::{CanyonError, CanyonResult};
use std::hash::{BuildHasher, RandomState};
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Every database made here starts with it.
pub const PREFIX: &str = "canyon_test_";
/// A test database older than this is a leftover of a crashed run.
pub const STALE_AFTER_SECS: u64 = 60 * 60;

// PostgreSQL's identifier limit, the smallest of the three backends
const MAX_NAME_LEN: usize = 63;
const MAX_TAG_LEN: usize = 24;
// Longest name `db_name` makes: prefix, secs (10), `_`, 8 hex, `_`, tag
const MAX_DB_NAME_LEN: usize = PREFIX.len() + 10 + 1 + 8 + 1 + MAX_TAG_LEN;
/// Longest suffix [`TestDb::schema`] takes (MySQL companions are `<name>_<suffix>`).
pub const MAX_SCHEMA_SUFFIX_LEN: usize = MAX_NAME_LEN - MAX_DB_NAME_LEN - 1;

/// `canyon_test_<created_at as 10 digits>_<suffix as 8 hex>_<tag>`, tag reduced to
/// `[a-z0-9_]` (`db` if empty), so [`created_at`] always reads it back.
pub fn db_name(tag: &str, created_at: u64, suffix: u32) -> String {
    let mut tag: String = tag
        .chars()
        .map(|c| c.to_ascii_lowercase())
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .take(MAX_TAG_LEN)
        .collect();
    if tag.is_empty() {
        tag.push_str("db");
    }
    let mut name = format!("{PREFIX}{created_at:010}_{suffix:08x}_{tag}");
    name.truncate(MAX_NAME_LEN);
    name
}

/// The creation time of a name with the exact shape [`db_name`] makes, `None` for any other.
pub fn created_at(name: &str) -> Option<u64> {
    let mut parts = name.strip_prefix(PREFIX)?.splitn(3, '_');
    let secs = parts
        .next()
        .filter(|s| s.len() == 10 && s.bytes().all(|b| b.is_ascii_digit()))?;
    parts
        .next()
        .filter(|h| h.len() == 8 && h.bytes().all(|b| b.is_ascii_hexdigit()))?;
    parts.next().filter(|tag| !tag.is_empty())?;
    secs.parse().ok()
}

/// `true` for a name made by [`db_name`] more than [`STALE_AFTER_SECS`] before `now`.
pub fn is_stale(name: &str, now: u64) -> bool {
    created_at(name).is_some_and(|t| now.saturating_sub(t) > STALE_AFTER_SECS)
}

/// `true` if `name` starts with [`PREFIX`], is at most 63 bytes and only has `[a-z0-9_]`.
/// Only such names are ever written into SQL here.
pub fn is_safe_name(name: &str) -> bool {
    name.starts_with(PREFIX) && name.len() <= MAX_NAME_LEN && name.bytes().all(is_name_byte)
}

/// `true` if `suffix` is 1 to [`MAX_SCHEMA_SUFFIX_LEN`] characters of `[a-z0-9_]`,
/// the suffixes [`TestDb::schema`] takes.
pub fn is_valid_schema_suffix(suffix: &str) -> bool {
    !suffix.is_empty() && suffix.len() <= MAX_SCHEMA_SUFFIX_LEN && suffix.bytes().all(is_name_byte)
}

fn is_name_byte(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'
}

pub(crate) fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

pub(crate) fn random_suffix() -> u32 {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    RandomState::new().hash_one((n, SystemTime::now())) as u32
}

/// A database created for one test, on the server of `datasource`.
///
/// ```ignore
/// TestDb::with(datasource, "add_column", async |db| {
///     db.conn().execute("CREATE TABLE t (c INT)", &[]).await?;
///     Ok(())
/// })
/// .await
/// ```
pub struct TestDb {
    name: String,
    datasource: String,
    db_type: DatabaseType,
    conn: DatabaseConnector,
}

impl TestDb {
    /// Runs `body` on a new database and drops it afterwards, also when `body`
    /// returns an error or panics. `body`'s error wins over a cleanup error, which
    /// is then only printed.
    ///
    /// Dropping this future before it ends (a timeout or `select!` around it) skips
    /// the cleanup and leaves the database to [`sweep_stale`]: put timeouts inside `body`.
    pub async fn with<T>(
        datasource: &str,
        tag: &str,
        body: impl AsyncFnOnce(&TestDb) -> CanyonResult<T>,
    ) -> CanyonResult<T> {
        let db = TestDb::new(datasource, tag).await?;
        let outcome = AssertUnwindSafe(body(&db)).catch_unwind().await;
        let cleaned = db.cleanup().await;
        if let (Err(error), false) = (&cleaned, matches!(outcome, Ok(Ok(_)))) {
            eprintln!("dropping test database on `{datasource}` failed: {error}");
        }
        match outcome {
            Ok(result) => {
                let value = result?;
                cleaned?;
                Ok(value)
            }
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    /// Creates an empty database next to the datasource's one and connects to it.
    /// Drops stale leftovers first. Prefer [`TestDb::with`]: after `new`, a failing
    /// test leaves the database behind until [`sweep_stale`] finds it.
    pub async fn new(datasource: &str, tag: &str) -> CanyonResult<Self> {
        // Housekeeping only: a failed sweep must not fail the test
        if let Err(error) = sweep_stale(datasource).await {
            eprintln!("sweeping stale test databases on `{datasource}` failed: {error}");
        }

        let canyon = Canyon::instance()?;
        let db_type = canyon.get_connection(datasource)?.get_db_type();
        let mut config = canyon
            .find_datasource_by_name_or_default(datasource)?
            .clone();
        let name = db_name(tag, now_secs(), random_suffix());
        config.properties.db_name = name.clone();
        create_database(datasource, &name).await?;

        // The pools connect lazily: `SELECT 1` makes sure the database is reachable
        let connected = async {
            let conn = DatabaseConnector::new(&config).await?;
            conn.query_rows("SELECT 1", &[]).await?;
            Ok::<_, CanyonError>(conn)
        }
        .await;
        let conn = match connected {
            Ok(conn) => conn,
            Err(error) => {
                // Keep the connection error, the drop is best effort here
                if let Err(drop_error) = drop_database(datasource, &name).await {
                    eprintln!("dropping `{name}` on `{datasource}` failed: {drop_error}");
                }
                return Err(error);
            }
        };

        Ok(Self {
            name,
            datasource: datasource.to_owned(),
            db_type,
            conn,
        })
    }

    /// A connection pool to this test's database.
    pub fn conn(&self) -> &DatabaseConnector {
        &self.conn
    }

    /// This test's database name, `canyon_test_…`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The backend of the datasource this database lives on.
    pub fn db_type(&self) -> DatabaseType {
        self.db_type
    }

    /// A second schema for this test. On MySQL a schema is a database, so it gets
    /// this database's name as prefix and `cleanup` drops it too.
    ///
    /// # Panics
    /// If `suffix` isn't 1 to [`MAX_SCHEMA_SUFFIX_LEN`] characters of `[a-z0-9_]`.
    /// Checked on every backend, so a bad suffix fails before a MySQL run leaks it.
    pub fn schema(&self, suffix: &str) -> String {
        assert!(
            is_valid_schema_suffix(suffix),
            "schema suffix must be 1 to {MAX_SCHEMA_SUFFIX_LEN} characters of [a-z0-9_]: {suffix}"
        );
        match self.db_type {
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => format!("{}_{suffix}", self.name),
            #[allow(unreachable_patterns)]
            _ => suffix.to_owned(),
        }
    }

    /// Closes the connection pool and drops the database (and its MySQL companions).
    /// The database is dropped even when dropping a companion fails.
    pub async fn cleanup(self) -> CanyonResult<()> {
        let Self {
            name,
            datasource,
            db_type,
            conn,
        } = self;
        drop(conn);
        let companions = drop_companions(&datasource, &name, db_type).await;
        let main = drop_database(&datasource, &name).await;
        companions.and(main)
    }
}

// On MySQL `schema` makes databases named `<name>_<suffix>`; elsewhere schemas live
// inside the test database and go with it
#[cfg_attr(not(feature = "mysql"), allow(unused_variables))]
async fn drop_companions(datasource: &str, name: &str, db_type: DatabaseType) -> CanyonResult<()> {
    #[cfg(feature = "mysql")]
    if matches!(db_type, DatabaseType::MySQL) {
        let prefix = format!("{name}_");
        for companion in list_test_databases(datasource)
            .await?
            .into_iter()
            .filter(|n| n.starts_with(&prefix))
        {
            drop_database(datasource, &companion).await?;
        }
    }
    Ok(())
}

/// Names of every `canyon_test_*` database on the datasource's server.
pub async fn list_test_databases(datasource: &str) -> CanyonResult<Vec<String>> {
    let admin = Canyon::instance()?.get_connection(datasource)?;
    // `_` is a LIKE wildcard, so the pattern matches a bit more: `is_safe_name` filters it
    let pattern = format!("{PREFIX}%");
    let rows = admin
        .query_rows(list_sql(admin.get_db_type()), &[&pattern])
        .await?;
    Ok(names_from(rows)
        .into_iter()
        .filter(|name| is_safe_name(name))
        .collect())
}

/// Creates the database `name` on the datasource's server.
///
/// # Panics
/// If `name` doesn't pass [`is_safe_name`].
pub async fn create_database(datasource: &str, name: &str) -> CanyonResult<()> {
    let admin = Canyon::instance()?.get_connection(datasource)?;
    admin
        .execute(&create_sql(admin.get_db_type(), name), &[])
        .await
        .map(|_| ())
}

/// Drops the database `name` from the datasource's server, closing its connections first.
///
/// # Panics
/// If `name` doesn't pass [`is_safe_name`].
pub async fn drop_database(datasource: &str, name: &str) -> CanyonResult<()> {
    let admin = Canyon::instance()?.get_connection(datasource)?;
    admin
        .execute(&drop_sql(admin.get_db_type(), name), &[])
        .await
        .map(|_| ())
}

// Database names can't be bound parameters: only names made by `db_name` get here
fn create_sql(db_type: DatabaseType, name: &str) -> String {
    assert!(is_safe_name(name), "not a test database name: {name}");
    match db_type {
        #[cfg(feature = "postgres")]
        DatabaseType::PostgreSql => format!("CREATE DATABASE \"{name}\""),
        #[cfg(feature = "mssql")]
        DatabaseType::SqlServer => format!("CREATE DATABASE [{name}]"),
        #[cfg(feature = "mysql")]
        DatabaseType::MySQL => format!("CREATE DATABASE `{name}`"),
    }
}

fn drop_sql(db_type: DatabaseType, name: &str) -> String {
    assert!(is_safe_name(name), "not a test database name: {name}");
    match db_type {
        #[cfg(feature = "postgres")]
        DatabaseType::PostgreSql => format!("DROP DATABASE IF EXISTS \"{name}\" WITH (FORCE)"),
        // Two sessions dropping the same database kill each other (`ROLLBACK IMMEDIATE`),
        // so the drop holds a lock named after it; the one that waited finds it gone.
        // Applocks belong to the current database: this works because every drop runs
        // from the datasource's database (`master` in `tests/canyon.toml`)
        #[cfg(feature = "mssql")]
        DatabaseType::SqlServer => format!(
            "DECLARE @locked INT; \
             EXEC @locked = sp_getapplock @Resource = N'{name}', @LockMode = 'Exclusive', \
             @LockOwner = 'Session', @LockTimeout = 60000; \
             IF @locked < 0 BEGIN DECLARE @msg NVARCHAR(200) = \
             CONCAT(N'sp_getapplock returned ', @locked, N' for {name}'); \
             THROW 50000, @msg, 1; END; \
             BEGIN TRY IF DB_ID(N'{name}') IS NOT NULL BEGIN \
             ALTER DATABASE [{name}] SET SINGLE_USER WITH ROLLBACK IMMEDIATE; \
             DROP DATABASE [{name}]; END END TRY \
             BEGIN CATCH EXEC sp_releaseapplock @Resource = N'{name}', @LockOwner = 'Session'; \
             THROW; END CATCH; \
             EXEC sp_releaseapplock @Resource = N'{name}', @LockOwner = 'Session';"
        ),
        #[cfg(feature = "mysql")]
        DatabaseType::MySQL => format!("DROP DATABASE IF EXISTS `{name}`"),
    }
}

fn list_sql(db_type: DatabaseType) -> &'static str {
    match db_type {
        #[cfg(feature = "postgres")]
        DatabaseType::PostgreSql => "SELECT datname FROM pg_database WHERE datname LIKE $1",
        #[cfg(feature = "mssql")]
        DatabaseType::SqlServer => "SELECT name FROM sys.databases WHERE name LIKE @P1",
        #[cfg(feature = "mysql")]
        DatabaseType::MySQL => {
            "SELECT SCHEMA_NAME FROM information_schema.SCHEMATA WHERE SCHEMA_NAME LIKE ?"
        }
    }
}

fn names_from(rows: CanyonRows) -> Vec<String> {
    match rows {
        #[cfg(feature = "postgres")]
        CanyonRows::Postgres(rows) => rows.iter().map(|row| row.get::<usize, String>(0)).collect(),
        #[cfg(feature = "mssql")]
        CanyonRows::Tiberius(rows) => rows
            .iter()
            .filter_map(|row| row.get::<&str, usize>(0).map(str::to_owned))
            .collect(),
        #[cfg(feature = "mysql")]
        CanyonRows::MySQL(rows) => rows
            .iter()
            .filter_map(|row| row.get::<String, usize>(0))
            .collect(),
    }
}

/// Drops the `canyon_test_*` databases older than [`STALE_AFTER_SECS`] and returns how
/// many it dropped. A drop that fails is printed and skipped: the next sweep tries again.
pub async fn sweep_stale(datasource: &str) -> CanyonResult<usize> {
    let now = now_secs();
    let mut dropped = 0;
    for name in list_test_databases(datasource).await? {
        if is_stale(&name, now) {
            match drop_database(datasource, &name).await {
                Ok(()) => dropped += 1,
                Err(error) => {
                    eprintln!("dropping stale `{name}` on `{datasource}` failed: {error}")
                }
            }
        }
    }
    Ok(dropped)
}
