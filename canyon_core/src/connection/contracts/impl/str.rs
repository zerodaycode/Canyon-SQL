//! This module contains the implementation of the `DbConnection` trait for the `&str` type.

macro_rules! impl_db_connection {
    ($type:ty) => {
        impl crate::connection::contracts::DbConnection for $type {
            async fn query_rows<'a>(
                &self,
                stmt: &str,
                params: &[&'a dyn crate::query::parameters::QueryParameter],
            ) -> Result<crate::rows::CanyonRows, Box<(dyn std::error::Error + Send + Sync)>> {
                let conn = crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.query_rows(stmt, params).await
            }

            async fn query<'a, S, R>(
                &self,
                stmt: S,
                params: &[&'a (dyn crate::query::parameters::QueryParameter)],
            ) -> Result<Vec<R>, Box<(dyn std::error::Error + Send + Sync)>>
            where
                S: AsRef<str> + Send,
                R: crate::mapper::RowMapper,
                Vec<R>: std::iter::FromIterator<<R as crate::mapper::RowMapper>::Output>,
            {
                let conn = crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.query(stmt, params).await
            }

            async fn query_one<'a, R>(
                &self,
                stmt: &str,
                params: &[&'a dyn crate::query::parameters::QueryParameter],
            ) -> Result<Option<R::Output>, Box<(dyn std::error::Error + Send + Sync)>>
            where
                R: crate::mapper::RowMapper,
            {
                let conn = crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.query_one::<R>(stmt, params).await
            }

            async fn query_one_for<'a, T: crate::rows::FromSqlOwnedValue<T>>(
                &self,
                stmt: &str,
                params: &[&'a dyn crate::query::parameters::QueryParameter],
            ) -> Result<T, Box<(dyn std::error::Error + Send + Sync)>> {
                let conn = crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.query_one_for(stmt, params).await
            }

            async fn execute<'a>(
                &self,
                stmt: &str,
                params: &[&'a dyn crate::query::parameters::QueryParameter],
            ) -> Result<u64, Box<(dyn std::error::Error + Send + Sync)>> {
                let conn = crate::connection::Canyon::instance()?.get_connection(self)?;
                conn.execute(stmt, params).await
            }

            fn get_database_type(
                &self,
            ) -> Result<
                crate::connection::database_type::DatabaseType,
                Box<(dyn std::error::Error + Send + Sync)>,
            > {
                Ok(crate::connection::Canyon::instance()?
                    .find_datasource_by_name_or_default(self)?
                    .get_db_type())
            }
        }
    };
}
