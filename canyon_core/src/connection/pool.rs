// src/pool.rs
// High-performance hybrid pool for Canyon-SQL
// bb8 for Postgres + MSSQL; native mysql_async::Pool for MySQL.

use std::sync::Arc;
//use async_trait::async_trait;
use bb8::{self, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use bb8_tiberius::ConnectionManager as TiberiusConnectionManager;
use mysql_async::{self, Pool as MySqlPool};
use tiberius::Config;
use tokio_postgres::NoTls;

// ============================
// === Type Aliases & Enums ===
// ============================

type PgManager = PostgresConnectionManager<NoTls>;
type PgPooled<'a> = PooledConnection<'a, PgManager>;

type MsManager = TiberiusConnectionManager;
type MsPooled<'a> = PooledConnection<'a, MsManager>;

/// Represents a checked-out database connection
pub enum CanyonConnection<'a> {
    Postgres(PgPooled<'a>),
    Mssql(MsPooled<'a>),
    Mysql(mysql_async::Conn),
}

/// Represents a Canyon-SQL connection pool (hybrid)
#[derive(Clone)]
pub enum CanyonPool {
    Postgres(Arc<bb8::Pool<PgManager>>),
    Mssql(Arc<bb8::Pool<MsManager>>),
    Mysql(Arc<MySqlPool>), // TODO: this one satisfies Send + Sync, so no need to wrap it in Arc<Mutex>>
}

// ==========================
// === Pool Constructors ===
// ==========================


impl CanyonPool {
    /// Create a Postgres pool
    pub async fn new_postgres(
        config: tokio_postgres::Config,
        max_size: u32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let manager = PostgresConnectionManager::new(config, NoTls);
        let pool = bb8::Pool::builder().max_size(max_size).build(manager).await?;
        Ok(Self::Postgres(Arc::new(pool)))
    }

    /// Create an MSSQL pool
    pub async fn new_mssql(
        config: Config,
        max_size: u32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let manager = TiberiusConnectionManager::new(config);
        let pool = bb8::Pool::builder().max_size(max_size).build(manager).await?;
        Ok(Self::Mssql(Arc::new(pool)))
    }

    /// Create a MySQL pool (native mysql_async::Pool)
    pub fn new_mysql(opts: mysql_async::Opts, max_size: usize) -> Self {
        let pool = MySqlPool::new(opts);
        Self::Mysql(Arc::new(pool))
    }

    /// Fetch a connection from the pool
    pub async fn get_conn<'a>(
        &'a self,
    ) -> Result<CanyonConnection<'a>, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            Self::Postgres(p) => {
                let c = p.get().await?;
                Ok(CanyonConnection::Postgres(c))
            }
            Self::Mssql(p) => {
                let c = p.get().await?;
                Ok(CanyonConnection::Mssql(c))
            }
            Self::Mysql(p) => {
                let c = p.get_conn().await?;
                Ok(CanyonConnection::Mysql(c))
            }
        }
    }
}