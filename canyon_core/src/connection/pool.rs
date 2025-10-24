use crate::connection::database_type::DatabaseType;
use crate::connection::datasources::DatasourceConfig;
use crate::connection::db_connector::DatabaseConnection;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// A simple, efficient connection pool for Canyon-SQL
///
/// This pool maintains a collection of database connections that can be
/// reused across multiple operations, significantly improving performance
/// by avoiding the overhead of creating new connections for each query.
pub struct ConnectionPool {
    /// The actual database connections in the pool
    connections: VecDeque<DatabaseConnection>,
    /// Maximum number of connections in the pool
    max_size: usize,
    /// Minimum number of connections to keep in the pool (currently unused)
    #[allow(dead_code)]
    min_size: usize,
    /// Database type for this pool
    db_type: DatabaseType,
    /// Connection factory function
    factory:
        Box<dyn Fn() -> Result<DatabaseConnection, Box<dyn Error + Send + Sync>> + Send + Sync>,
}

impl ConnectionPool {
    /// Creates a new connection pool
    pub fn new(
        db_type: DatabaseType,
        factory: impl Fn() -> Result<DatabaseConnection, Box<dyn Error + Send + Sync>>
        + Send
        + Sync
        + 'static,
        min_size: usize,
        max_size: usize,
    ) -> Self {
        Self {
            connections: VecDeque::new(),
            max_size,
            min_size,
            db_type,
            factory: Box::new(factory),
        }
    }

    /// Gets a connection from the pool
    ///
    /// If a connection is available, it's returned immediately.
    /// If no connections are available and the pool hasn't reached max_size,
    /// a new connection is created.
    /// If the pool is at max_size, this will wait for a connection to become available.
    pub async fn get_connection(
        &mut self,
    ) -> Result<DatabaseConnection, Box<dyn Error + Send + Sync>> {
        // Try to get an existing connection
        if let Some(conn) = self.connections.pop_front() {
            return Ok(conn);
        }

        // Create a new connection if we haven't reached max_size
        if self.connections.len() < self.max_size {
            return (self.factory)();
        }

        // Wait for a connection to become available
        // This is a simple implementation - in production you might want more sophisticated waiting
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Use Box::pin to avoid recursion issues
        Box::pin(self.get_connection()).await
    }

    /// Returns a connection to the pool
    ///
    /// If the pool is at max_size, the connection is dropped.
    /// Otherwise, it's added back to the pool for reuse.
    pub fn return_connection(&mut self, conn: DatabaseConnection) {
        if self.connections.len() < self.max_size {
            self.connections.push_back(conn);
        }
        // If pool is full, the connection is dropped
    }

    /// Gets the database type for this pool
    pub fn db_type(&self) -> DatabaseType {
        self.db_type
    }

    /// Gets the current pool size
    pub fn size(&self) -> usize {
        self.connections.len()
    }

    /// Gets the maximum pool size
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

/// A wrapper around a pooled connection that automatically returns it to the pool when dropped
pub struct PooledConnection {
    connection: Option<DatabaseConnection>,
    pool: Arc<Mutex<ConnectionPool>>,
}

impl PooledConnection {
    /// Creates a new pooled connection wrapper
    pub async fn new(
        pool: Arc<Mutex<ConnectionPool>>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let connection = {
            let mut pool_guard = pool.lock().await;
            pool_guard.get_connection().await?
        };

        Ok(Self {
            connection: Some(connection),
            pool,
        })
    }

    /// Gets a reference to the underlying connection
    pub fn connection(&self) -> &DatabaseConnection {
        self.connection.as_ref().unwrap()
    }

    /// Gets a mutable reference to the underlying connection
    pub fn connection_mut(&mut self) -> &mut DatabaseConnection {
        self.connection.as_mut().unwrap()
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        // Return the connection to the pool when this wrapper is dropped
        if let Some(conn) = self.connection.take() {
            // We can't use async in Drop, so we spawn a task to return the connection
            let pool = self.pool.clone();
            tokio::spawn(async move {
                let mut pool_guard = pool.lock().await;
                pool_guard.return_connection(conn);
            });
        }
    }
}

/// Global connection pool manager
pub struct PoolManager {
    pools: HashMap<String, Arc<Mutex<ConnectionPool>>>,
}

impl PoolManager {
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
        }
    }

    /// Creates a connection pool for a datasource
    pub async fn create_pool(
        &mut self,
        name: &str,
        datasource: &DatasourceConfig,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let db_type = datasource.get_db_type();

        // Create a factory function for this datasource
        let factory = {
            let datasource = datasource.clone();
            move || {
                // Use tokio::spawn to handle the async DatabaseConnection::new
                let rt = tokio::runtime::Handle::current();
                rt.block_on(DatabaseConnection::new(&datasource))
            }
        };

        let pool = ConnectionPool::new(
            db_type, factory, 2,  // min_size
            10, // max_size
        );

        self.pools
            .insert(name.to_string(), Arc::new(Mutex::new(pool)));
        Ok(())
    }

    /// Gets a pooled connection by name
    pub async fn get_connection(
        &self,
        name: &str,
    ) -> Result<PooledConnection, Box<dyn Error + Send + Sync>> {
        let pool = self
            .pools
            .get(name)
            .ok_or_else(|| format!("Pool '{}' not found", name))?;

        PooledConnection::new(pool.clone()).await
    }

    /// Gets the default connection pool
    pub async fn get_default_connection(
        &self,
    ) -> Result<PooledConnection, Box<dyn Error + Send + Sync>> {
        let pool = self
            .pools
            .values()
            .next()
            .ok_or("No connection pools available")?;

        PooledConnection::new(pool.clone()).await
    }

    /// Checks if a pool exists for the given name
    pub fn has_pool(&self, name: &str) -> bool {
        self.pools.contains_key(name)
    }
}

// Global pool manager instance
static POOL_MANAGER: std::sync::OnceLock<Arc<Mutex<PoolManager>>> = std::sync::OnceLock::new();

/// Gets the global pool manager instance
pub fn get_pool_manager() -> Arc<Mutex<PoolManager>> {
    POOL_MANAGER
        .get_or_init(|| Arc::new(Mutex::new(PoolManager::new())))
        .clone()
}

// Implement DbConnection trait for PooledConnection
impl crate::connection::contracts::DbConnection for PooledConnection {
    fn query_rows(
        &self,
        stmt: &str,
        params: &[&dyn crate::query::parameters::QueryParameter],
    ) -> impl std::future::Future<
        Output = Result<crate::rows::CanyonRows, Box<dyn Error + Send + Sync>>,
    > + Send {
        let conn = self.connection();
        async move { conn.query_rows(stmt, params).await }
    }

    fn query<S, R>(
        &self,
        stmt: S,
        params: &[&dyn crate::query::parameters::QueryParameter],
    ) -> impl std::future::Future<Output = Result<Vec<R>, Box<dyn Error + Send + Sync>>> + Send
    where
        S: AsRef<str> + Send,
        R: crate::mapper::RowMapper,
        Vec<R>: std::iter::FromIterator<<R as crate::mapper::RowMapper>::Output>,
    {
        let conn = self.connection();
        async move { conn.query(stmt, params).await }
    }

    fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&dyn crate::query::parameters::QueryParameter],
    ) -> impl std::future::Future<Output = Result<Option<R::Output>, Box<dyn Error + Send + Sync>>> + Send
    where
        R: crate::mapper::RowMapper,
    {
        let conn = self.connection();
        async move { conn.query_one::<R>(stmt, params).await }
    }

    fn query_one_for<T: crate::rows::FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&dyn crate::query::parameters::QueryParameter],
    ) -> impl std::future::Future<Output = Result<T, Box<dyn Error + Send + Sync>>> + Send {
        let conn = self.connection();
        async move { conn.query_one_for(stmt, params).await }
    }

    fn execute(
        &self,
        stmt: &str,
        params: &[&dyn crate::query::parameters::QueryParameter],
    ) -> impl std::future::Future<Output = Result<u64, Box<dyn Error + Send + Sync>>> + Send {
        let conn = self.connection();
        async move { conn.execute(stmt, params).await }
    }

    fn get_database_type(
        &self,
    ) -> Result<crate::connection::database_type::DatabaseType, Box<dyn Error + Send + Sync>> {
        self.connection().get_database_type()
    }
}
