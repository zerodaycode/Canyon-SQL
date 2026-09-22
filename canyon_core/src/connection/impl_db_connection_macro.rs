//! This module contains macros for helping us to reduce boilerplate implementation code of the
//! [`crate::connection::DbConnection`]

#[macro_export]
macro_rules! impl_db_connection_for_db_connector {
    ($type:ty) => {
        impl $crate::connection::contracts::DbConnection for $type {
            async fn query_rows(
                &self,
                stmt: &str,
                params: &[&'_ dyn QueryParameter],
            ) -> $crate::error::CanyonResult<CanyonRows> {
                match self {
                    #[cfg(feature = "postgres")]
                    DatabaseConnector::Postgres(client) => client.query_rows(stmt, params).await,

                    #[cfg(feature = "mssql")]
                    DatabaseConnector::SqlServer(client) => client.query_rows(stmt, params).await,

                    #[cfg(feature = "mysql")]
                    DatabaseConnector::MySQL(client) => client.query_rows(stmt, params).await,
                }
            }

            async fn query<S, R>(
                &self,
                stmt: S,
                params: &[&'_ dyn QueryParameter],
            ) -> $crate::error::CanyonResult<Vec<R>>
            where
                S: AsRef<str> + Send,
                R: RowMapper,
                Vec<R>: FromIterator<<R as RowMapper>::Output>,
            {
                match self {
                    #[cfg(feature = "postgres")]
                    DatabaseConnector::Postgres(client) => client.query(stmt, params).await,

                    #[cfg(feature = "mssql")]
                    DatabaseConnector::SqlServer(client) => client.query(stmt, params).await,

                    #[cfg(feature = "mysql")]
                    DatabaseConnector::MySQL(client) => client.query(stmt, params).await,
                }
            }

            async fn query_one<R>(
                &self,
                stmt: &str,
                params: &[&'_ dyn QueryParameter],
            ) -> $crate::error::CanyonResult<Option<R::Output>>
            where
                R: RowMapper,
            {
                match self {
                    #[cfg(feature = "postgres")]
                    DatabaseConnector::Postgres(client) => {
                        client.query_one::<R>(stmt, params).await
                    }

                    #[cfg(feature = "mssql")]
                    DatabaseConnector::SqlServer(client) => {
                        client.query_one::<R>(stmt, params).await
                    }

                    #[cfg(feature = "mysql")]
                    DatabaseConnector::MySQL(client) => client.query_one::<R>(stmt, params).await,
                }
            }

            async fn query_one_for<T: FromSqlOwnedValue>(
                &self,
                stmt: &str,
                params: &[&'_ dyn QueryParameter],
            ) -> $crate::error::CanyonResult<T> {
                match self {
                    #[cfg(feature = "postgres")]
                    DatabaseConnector::Postgres(client) => client.query_one_for(stmt, params).await,

                    #[cfg(feature = "mssql")]
                    DatabaseConnector::SqlServer(client) => {
                        client.query_one_for(stmt, params).await
                    }

                    #[cfg(feature = "mysql")]
                    DatabaseConnector::MySQL(client) => client.query_one_for(stmt, params).await,
                }
            }

            async fn execute(
                &self,
                stmt: &str,
                params: &[&'_ dyn QueryParameter],
            ) -> $crate::error::CanyonResult<u64> {
                match self {
                    #[cfg(feature = "postgres")]
                    DatabaseConnector::Postgres(client) => client.execute(stmt, params).await,

                    #[cfg(feature = "mssql")]
                    DatabaseConnector::SqlServer(client) => client.execute(stmt, params).await,

                    #[cfg(feature = "mysql")]
                    DatabaseConnector::MySQL(client) => client.execute(stmt, params).await,
                }
            }

            fn get_database_type(&self) -> $crate::error::CanyonResult<DatabaseType> {
                Ok(self.get_db_type())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_db_connection_for_str {
    ($type:ty) => {
        impl $crate::connection::contracts::DbConnection for $type {
            async fn query_rows(
                &self,
                stmt: &str,
                params: &[&'_ dyn $crate::query::parameters::QueryParameter],
            ) -> $crate::error::CanyonResult<$crate::rows::CanyonRows> {
                let conn = $crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.query_rows(stmt, params).await
            }

            async fn query<S, R>(
                &self,
                stmt: S,
                params: &[&'_ dyn $crate::query::parameters::QueryParameter],
            ) -> $crate::error::CanyonResult<Vec<R>>
            where
                S: AsRef<str> + Send,
                R: $crate::mapper::RowMapper,
                Vec<R>: std::iter::FromIterator<<R as $crate::mapper::RowMapper>::Output>,
            {
                let conn = $crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.query(stmt, params).await
            }

            async fn query_one<R>(
                &self,
                stmt: &str,
                params: &[&'_ dyn $crate::query::parameters::QueryParameter],
            ) -> $crate::error::CanyonResult<Option<R::Output>>
            where
                R: $crate::mapper::RowMapper,
            {
                let conn = $crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.query_one::<R>(stmt, params).await
            }

            async fn query_one_for<T: $crate::rows::FromSqlOwnedValue>(
                &self,
                stmt: &str,
                params: &[&'_ dyn $crate::query::parameters::QueryParameter],
            ) -> $crate::error::CanyonResult<T> {
                let conn = $crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.query_one_for(stmt, params).await
            }

            async fn execute(
                &self,
                stmt: &str,
                params: &[&'_ dyn $crate::query::parameters::QueryParameter],
            ) -> $crate::error::CanyonResult<u64> {
                let conn = $crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.execute(stmt, params).await
            }

            fn get_database_type(
                &self,
            ) -> $crate::error::CanyonResult<$crate::connection::database_type::DatabaseType> {
                Ok($crate::connection::Canyon::instance()?
                    .find_datasource_by_name_or_default(self)?
                    .get_db_type())
            }
        }
    };
}
