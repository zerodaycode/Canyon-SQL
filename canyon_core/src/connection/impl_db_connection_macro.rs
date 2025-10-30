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
            ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
                db_conn_query_rows_impl(self, stmt, params).await
            }

            async fn query<S, R>(
                &self,
                stmt: S,
                params: &[&'_ dyn QueryParameter],
            ) -> Result<Vec<R>, Box<dyn Error + Send + Sync>>
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
            ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
            where
                R: RowMapper,
            {
                db_conn_query_one_impl::<R>(self, stmt, params).await
            }

            async fn query_one_for<T: FromSqlOwnedValue<T>>(
                &self,
                stmt: &str,
                params: &[&'_ dyn QueryParameter],
            ) -> Result<T, Box<dyn Error + Send + Sync>> {
                db_conn_query_one_for_impl::<T>(self, stmt, params).await
            }

            async fn execute(
                &self,
                stmt: &str,
                params: &[&'_ dyn QueryParameter],
            ) -> Result<u64, Box<dyn Error + Send + Sync>> {
                db_conn_execute_impl(self, stmt, params).await
            }

            fn get_database_type(&self) -> Result<DatabaseType, Box<dyn Error + Send + Sync>> {
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
            ) -> Result<$crate::rows::CanyonRows, Box<dyn std::error::Error + Send + Sync>> {
                let conn = $crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.query_rows(stmt, params).await
            }

            async fn query<S, R>(
                &self,
                stmt: S,
                params: &[&'_ dyn $crate::query::parameters::QueryParameter],
            ) -> Result<Vec<R>, Box<dyn std::error::Error + Send + Sync>>
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
            ) -> Result<Option<R::Output>, Box<dyn std::error::Error + Send + Sync>>
            where
                R: $crate::mapper::RowMapper,
            {
                let conn = $crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.query_one::<R>(stmt, params).await
            }

            async fn query_one_for<T: $crate::rows::FromSqlOwnedValue<T>>(
                &self,
                stmt: &str,
                params: &[&'_ dyn $crate::query::parameters::QueryParameter],
            ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
                let conn = $crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.query_one_for(stmt, params).await
            }

            async fn execute(
                &self,
                stmt: &str,
                params: &[&'_ dyn $crate::query::parameters::QueryParameter],
            ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
                let conn = $crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.execute(stmt, params).await
            }

            fn get_database_type(
                &self,
            ) -> Result<
                $crate::connection::database_type::DatabaseType,
                Box<dyn std::error::Error + Send + Sync>,
            > {
                Ok($crate::connection::Canyon::instance()?
                    .find_datasource_by_name_or_default(self)?
                    .get_db_type())
            }
        }
    };
}
