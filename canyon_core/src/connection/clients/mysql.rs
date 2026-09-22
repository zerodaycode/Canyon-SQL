use crate::connection::clients::mysql::mysql_query_launcher::{execute_query, generate_mysql_stmt};
use crate::connection::contracts::DbConnection;
use crate::connection::database_type::DatabaseType;
use crate::connection::datasources::DatasourceConfig;
use crate::error::{CanyonResult, ConnectionError, QueryError};
use crate::mapper::RowMapper;
use crate::rows::FromSqlOwnedValue;
use crate::{query::parameters::QueryParameter, rows::CanyonRows};
use mysql_async::Row;
use mysql_async::prelude::Query;
use mysql_common::constants::ColumnType;
use mysql_common::row;

/// A connection with a MySQL database.
pub struct MySQLConnector(mysql_async::Pool);

impl MySQLConnector {
    pub async fn new(config: &DatasourceConfig) -> CanyonResult<Self> {
        Ok(Self(__impl::load_mysql_config(config).await?))
    }
}

impl DbConnection for MySQLConnector {
    async fn query_rows(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> CanyonResult<CanyonRows> {
        Ok(CanyonRows::MySQL(execute_query(stmt, params, self).await?))
    }

    async fn query<S, R>(&self, stmt: S, params: &[&'_ dyn QueryParameter]) -> CanyonResult<Vec<R>>
    where
        S: AsRef<str> + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        execute_query(stmt, params, self)
            .await?
            .iter()
            .map(R::deserialize_mysql)
            .collect()
    }

    async fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> CanyonResult<Option<R::Output>>
    where
        R: RowMapper,
    {
        let result = execute_query(stmt, params, self).await?;

        match result.first() {
            Some(row) => Ok(Some(R::deserialize_mysql(row)?)),
            None => Ok(None),
        }
    }

    async fn query_one_for<T: FromSqlOwnedValue>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> CanyonResult<T> {
        execute_query(stmt, params, self)
            .await?
            .first()
            .ok_or(QueryError::NoRows)?
            .get_opt::<T, usize>(0)
            .ok_or(QueryError::NoColumns)?
            .map_err(|source| QueryError::mysql_value(source).into())
    }

    async fn execute(&self, stmt: &str, params: &[&'_ dyn QueryParameter]) -> CanyonResult<u64> {
        let mysql_connection = self.0.get_conn().await.map_err(ConnectionError::mysql)?;
        let mysql_stmt = generate_mysql_stmt(stmt, params)?;

        Ok(mysql_stmt
            .stmt
            .run(mysql_connection)
            .await
            .map_err(QueryError::mysql)?
            .affected_rows())
    }

    fn get_database_type(&self) -> CanyonResult<DatabaseType> {
        Ok(DatabaseType::MySQL)
    }
}

pub(crate) mod mysql_query_launcher {
    use super::*;

    use mysql_async::{QueryWithParams, Value};
    use std::sync::Arc;

    pub(crate) struct MySqlGeneratedStmt {
        pub(crate) stmt: QueryWithParams<String, Vec<Value>>,
        returns_last_insert_id: bool,
    }

    pub(crate) async fn execute_query<S>(
        stmt: S,
        params: &[&'_ dyn QueryParameter],
        connector: &MySQLConnector,
    ) -> CanyonResult<Vec<Row>>
    where
        S: AsRef<str> + Send,
    {
        let mysql_connection = connector
            .0
            .get_conn()
            .await
            .map_err(ConnectionError::mysql)?;
        let mysql_stmt = generate_mysql_stmt(stmt.as_ref(), params)?;

        let returns_last_insert_id = mysql_stmt.returns_last_insert_id;
        let mut query_result = mysql_stmt
            .stmt
            .run(mysql_connection)
            .await
            .map_err(QueryError::mysql)?;

        if returns_last_insert_id {
            let last_insert_id = query_result.last_insert_id().ok_or(QueryError::NoRows)?;

            return Ok(vec![row::new_row(
                vec![Value::UInt(last_insert_id)],
                Arc::new([mysql_async::Column::new(ColumnType::MYSQL_TYPE_LONGLONG)]),
            )]);
        }

        query_result
            .collect::<Row>()
            .await
            .map_err(|source| QueryError::mysql(source).into())
    }

    pub(crate) fn generate_mysql_stmt(
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> CanyonResult<MySqlGeneratedStmt> {
        let params = params.iter().map(|param| param.as_mysql_param()).collect();

        Ok(MySqlGeneratedStmt {
            stmt: QueryWithParams {
                query: stmt.to_owned(),
                params,
            },
            returns_last_insert_id: is_insert_statement(stmt),
        })
    }

    fn is_insert_statement(stmt: &str) -> bool {
        stmt.trim_start()
            .split_once(char::is_whitespace)
            .map_or(stmt.trim_start(), |(keyword, _)| keyword)
            .eq_ignore_ascii_case("INSERT")
    }

    #[cfg(test)]
    mod tests {
        use super::is_insert_statement;

        #[test]
        fn detects_insert_statements() {
            assert!(is_insert_statement(
                "INSERT INTO `users` (`name`) VALUES (?)"
            ));
            assert!(is_insert_statement(
                "  \n INSERT INTO `users` (`name`) VALUES (?)"
            ));
            assert!(is_insert_statement(
                "insert into `users` (`name`) values (?)"
            ));
        }

        #[test]
        fn does_not_treat_other_statements_as_inserts() {
            assert!(!is_insert_statement("SELECT * FROM `users`"));
            assert!(!is_insert_statement(
                "UPDATE `users` SET `name` = ? WHERE `id` = ?"
            ));
            assert!(!is_insert_statement("DELETE FROM `users` WHERE `id` = ?"));
        }
    }
}

pub(crate) mod __impl {
    use crate::connection::database_type::DatabaseType;
    use crate::connection::datasources::{Auth, DatasourceConfig, MySQLAuth};
    use crate::error::{CanyonResult, ConfigurationError, ConnectionError};
    use mysql_async::Pool;

    pub(crate) async fn load_mysql_config(datasource: &DatasourceConfig) -> CanyonResult<Pool> {
        let (user, password) = extract_mysql_auth(&datasource.auth)?;

        // TODO: pool constraints must be obtained from the datasource configuration.
        let pool_constraints = mysql_async::PoolConstraints::new(2, 10).ok_or(
            ConnectionError::InvalidPoolConfiguration {
                backend: DatabaseType::MySQL,
            },
        )?;

        let mysql_opts_builder = mysql_async::OptsBuilder::default()
            .pool_opts(mysql_async::PoolOpts::default().with_constraints(pool_constraints))
            .user(Some(user))
            .pass(Some(password))
            .db_name(Some(&datasource.properties.db_name))
            .ip_or_hostname(&datasource.properties.host)
            .tcp_port(datasource.get_port_or_default_by_db());

        Ok(mysql_async::Pool::new(mysql_opts_builder))
    }

    pub(crate) fn extract_mysql_auth(auth: &Auth) -> CanyonResult<(&str, &str)> {
        match auth {
            Auth::MySQL(MySQLAuth::Basic { username, password }) => Ok((username, password)),
            #[cfg(any(feature = "postgres", feature = "mssql"))]
            _ => Err(ConfigurationError::InvalidAuthentication {
                backend: DatabaseType::MySQL,
            }
            .into()),
        }
    }
}
