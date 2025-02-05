#[cfg(feature = "mysql")]
use mysql_async::Pool;
use std::error::Error;
use std::fmt::Display;
use std::future::Future;
use crate::connection::database_type::DatabaseType;
use crate::connection::db_connector::DbConnection;
use crate::{query_parameters::QueryParameter, rows::CanyonRows};
use mysql_async::Row;
use mysql_common::constants::ColumnType;
use mysql_common::row;
use crate::mapper::RowMapper;

/// A connection with a `Mysql` database
#[cfg(feature = "mysql")]
pub struct MysqlConnection {
    pub client: Pool,
}

impl DbConnection for MysqlConnection {
    fn query<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn Error + Sync + Send)>>> + Send
    {
        mysql_query_launcher::query(stmt, params, self)
    }

    fn query_rows<'a, S, Z, R: RowMapper<R>>(&self, stmt: S, params: Z) -> impl Future<Output=Result<Vec<R>, Box<(dyn Error + Sync + Send + 'a)>>> + Send
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a
    {
        async move { todo!() }
    }

    fn query_one<'a, R: RowMapper<R>>(&self, stmt: &str, params: &[&'a dyn QueryParameter<'a>]) 
        -> impl Future<Output=Result<Option<R>, Box<(dyn Error + Sync + Send)>>> + Send
    {
        mysql_query_launcher::query_one(stmt, params, self)
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Sync + Send)>> {
        Ok(DatabaseType::MySQL)
    }
}

#[cfg(feature = "mysql")]
pub(crate) mod mysql_query_launcher {
    #[cfg(feature = "mysql")]
    pub const DETECT_PARAMS_IN_QUERY: &str = r"\$([\d])+";
    #[cfg(feature = "mysql")]
    pub const DETECT_QUOTE_IN_QUERY: &str = r#"\"|\\"#;

    use super::*;

    use mysql_async::prelude::Query;
    use mysql_async::QueryWithParams;
    use mysql_async::Value;
    use regex::Regex;
    use std::sync::Arc;

    #[inline(always)] // TODO: very provisional implementation! care!
    // TODO: would be better to launch a simple query for the last id?
    pub(crate) async fn query_one<'a, R: RowMapper<R>>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &MysqlConnection,
    ) -> Result<Option<R>, Box<(dyn Error + Sync + Send)>> {
        Ok(query(stmt, params, conn)
            .await?
            .first_row()
        )
    }

    #[inline(always)] 
    pub(crate) async fn query<'a>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &MysqlConnection,
    ) -> Result<CanyonRows, Box<(dyn Error + Sync + Send)>> {
        let mysql_connection = conn.client.get_conn().await?;

        let stmt_with_escape_characters = regex::escape(stmt);
        let query_string =
            Regex::new(DETECT_PARAMS_IN_QUERY)?.replace_all(&stmt_with_escape_characters, "?");

        let mut query_string = Regex::new(DETECT_QUOTE_IN_QUERY)?
            .replace_all(&query_string, "")
            .to_string();

        let mut is_insert = false;
        // TODO: take care of this ugly replace for the concrete client syntax by using canyon
        // Query
        if let Some(index_start_clausule_returning) = query_string.find(" RETURNING") {
            query_string.truncate(index_start_clausule_returning);
            is_insert = true;
        }

        let params_query: Vec<Value> =
            reorder_params(stmt, params, |f| (*f).as_mysql_param().to_value());

        let query_with_params = QueryWithParams {
            query: query_string,
            params: params_query,
        };

        let mut query_result = query_with_params
            .run(mysql_connection)
            .await
            .expect("Error executing query in mysql");

        let result_rows = if is_insert {
            let last_insert = query_result
                .last_insert_id()
                .map(Value::UInt)
                .expect("Error getting pk id in insert");

            vec![row::new_row(
                vec![last_insert],
                Arc::new([mysql_async::Column::new(ColumnType::MYSQL_TYPE_UNKNOWN)]),
            )]
        } else {
            query_result
                .collect::<Row>()
                .await
                .expect("Error resolved trait FromRow in mysql")
        };
        let a = CanyonRows::MySQL(result_rows);
        Ok(a)
    }
}

#[cfg(feature = "mysql")]
fn reorder_params<T>(
    stmt: &str,
    params: &[&'_ dyn QueryParameter<'_>],
    fn_parser: impl Fn(&&dyn QueryParameter<'_>) -> T,
) -> Vec<T> {
    use mysql_query_launcher::DETECT_PARAMS_IN_QUERY;

    let mut ordered_params = vec![];
    let rg = regex::Regex::new(DETECT_PARAMS_IN_QUERY)
        .expect("Error create regex with detect params pattern expression");

    for positional_param in rg.find_iter(stmt) {
        let pp: &str = positional_param.as_str();
        let pp_index = pp[1..] // param $1 -> get 1
            .parse::<usize>()
            .expect("Error parse mapped parameter to usized.")
            - 1;

        let element = params
            .get(pp_index)
            .expect("Error obtaining the element of the mapping against parameters.");
        ordered_params.push(fn_parser(element));
    }

    ordered_params
}
