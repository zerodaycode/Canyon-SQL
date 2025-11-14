use crate::connection::clients::mysql::mysql_query_launcher::{execute_query, generate_mysql_stmt};
use crate::connection::contracts::DbConnection;
use crate::connection::database_type::DatabaseType;
use crate::connection::datasources::DatasourceConfig;
use crate::mapper::RowMapper;
use crate::rows::FromSqlOwnedValue;
use crate::{query::parameters::QueryParameter, rows::CanyonRows};
use mysql_async::Row;
use mysql_async::prelude::Query;
use mysql_common::constants::ColumnType;
use mysql_common::row;
use std::error::Error;

/// A connection with a `Mysql` database
pub struct MySQLConnector(mysql_async::Pool);

impl MySQLConnector {
    pub async fn new(config: &DatasourceConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self(__impl::load_mysql_config(config).await?))
    }
}

impl DbConnection for MySQLConnector {
    async fn query_rows(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
        Ok(CanyonRows::MySQL(execute_query(stmt, params, self).await?))
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
        Ok(execute_query(stmt, params, self)
            .await?
            .iter()
            .flat_map(|row| R::deserialize_mysql(row))
            .collect())
    }

    async fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
    where
        R: RowMapper,
    {
        let result = execute_query(stmt, params, self).await?;

        match result.first() {
            Some(r) => Ok(Some(R::deserialize_mysql(r)?)),
            None => Ok(None),
        }
    }

    async fn query_one_for<T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        Ok(execute_query(stmt, params, self)
            .await?
            .first()
            .ok_or_else(|| format!("Failure executing 'query_one_for' while retrieving the first row with stmt: {:?}", stmt))?
            .get::<T, usize>(0)
            .ok_or_else(|| format!("Failure executing 'query_one_for' while retrieving the first column value on the first row with stmt: {:?}", stmt))?
        )
    }

    async fn execute(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let mysql_connection = self.0.get_conn().await?;
        let mysql_stmt = generate_mysql_stmt(stmt.as_ref(), params)?;

        Ok(mysql_stmt.run(mysql_connection).await?.affected_rows())
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<dyn Error + Send + Sync>> {
        Ok(DatabaseType::MySQL)
    }
}

pub(crate) mod mysql_query_launcher {
    pub const DETECT_PARAMS_IN_QUERY: &str = r"\$([\d])+";
    pub const DETECT_QUOTE_IN_QUERY: &str = r#"\"|\\"#;

    use super::*;

    use mysql_async::QueryWithParams;
    use mysql_async::Value;
    use mysql_async::prelude::Query;
    use regex::Regex;
    use std::sync::Arc;

    pub(crate) async fn execute_query<S>(
        stmt: S,
        params: &[&'_ dyn QueryParameter],
        conn: &MySQLConnector,
    ) -> Result<Vec<Row>, Box<dyn Error + Send + Sync>>
    where
        S: AsRef<str> + Send,
    {
        let mysql_connection = conn.0.get_conn().await?;
        let is_insert = stmt.as_ref().find(" RETURNING");
        let mysql_stmt = generate_mysql_stmt(stmt.as_ref(), params)?;

        let mut query_result = mysql_stmt.run(mysql_connection).await?;
        let result_rows = if is_insert.is_some() {
            let last_insert = query_result
                .last_insert_id()
                .map(Value::UInt)
                .ok_or("MySQL: Error getting the id in insert")?;

            vec![row::new_row(
                vec![last_insert],
                Arc::new([mysql_async::Column::new(ColumnType::MYSQL_TYPE_UNKNOWN)]),
            )]
        } else {
            query_result.collect::<Row>().await?
        };

        Ok(result_rows)
    }

    pub(crate) fn generate_mysql_stmt(
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<QueryWithParams<String, Vec<Value>>, Box<dyn Error + Send + Sync>> {
        let stmt_with_escape_characters = regex::escape(stmt);
        let query_string =
            Regex::new(DETECT_PARAMS_IN_QUERY)?.replace_all(&stmt_with_escape_characters, "?");

        let mut query_string = Regex::new(DETECT_QUOTE_IN_QUERY)?
            .replace_all(&query_string, "")
            .to_string();

        if let Some(index_start_clausule_returning) = query_string.find(" RETURNING") {
            query_string.truncate(index_start_clausule_returning);
        }

        let params_query: Vec<Value> =
            reorder_params(stmt, params, |f| (*f).as_mysql_param().to_value())?;

        Ok(QueryWithParams {
            query: query_string,
            params: params_query,
        })
    }

    fn reorder_params<T>(
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
        fn_parser: impl Fn(&&dyn QueryParameter) -> T,
    ) -> Result<Vec<T>, Box<dyn Error + Send + Sync>> {
        use mysql_query_launcher::DETECT_PARAMS_IN_QUERY;

        let mut ordered_params = vec![];
        let rg = Regex::new(DETECT_PARAMS_IN_QUERY)
            .expect("Error create regex with detect params pattern expression");

        for positional_param in rg.find_iter(stmt) {
            let pp: &str = positional_param.as_str();
            let pp_index = pp[1..] // param $1 -> get 1
                .parse::<usize>()?
                - 1;

            let element = params
                .get(pp_index)
                .expect("Error obtaining the element of the mapping against parameters.");
            ordered_params.push(fn_parser(element));
        }

        Ok(ordered_params)
    }
}

pub(crate) mod __impl {
    use crate::connection::datasources::{Auth, DatasourceConfig, MySQLAuth};
    use mysql_async::Pool;
    use std::error::Error;

    pub(crate) async fn load_mysql_config(
        datasource: &DatasourceConfig,
    ) -> Result<Pool, Box<dyn Error + Send + Sync>> {
        let (user, password) = extract_mysql_auth(&datasource.auth)?;

        // TODO: the pool constrains must be adquired from the datasource config
        let pool_constraints =
            mysql_async::PoolConstraints::new(2, 10).ok_or("Failure launching the MySQL pool")?;

        let mysql_opts_builder = mysql_async::OptsBuilder::default()
            .pool_opts(mysql_async::PoolOpts::default().with_constraints(pool_constraints))
            .user(Some(user))
            .pass(Some(password))
            .db_name(Some(&datasource.properties.db_name))
            .ip_or_hostname(&datasource.properties.host)
            .tcp_port(datasource.get_port_or_default_by_db());

        Ok(mysql_async::Pool::new(mysql_opts_builder))
    }

    pub(crate) fn extract_mysql_auth(
        auth: &Auth,
    ) -> Result<(&str, &str), Box<dyn std::error::Error + Send + Sync>> {
        match auth {
            Auth::MySQL(mysql_auth) => match mysql_auth {
                MySQLAuth::Basic { username, password } => Ok((username, password)),
            },
            #[cfg(any(feature = "postgres", feature = "mssql"))]
            _ => Err("Invalid auth configuration for a MySQL datasource.".into()),
        }
    }
}
