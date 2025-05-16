use crate::mapper::RowMapper;
use crate::rows::FromSqlOwnedValue;
use crate::{query::parameters::QueryParameter, rows::CanyonRows};
#[cfg(feature = "mysql")]
use mysql_async::Pool;
use mysql_async::Row;
use mysql_common::constants::ColumnType;
use mysql_common::row;
use std::error::Error;

/// A connection with a `Mysql` database
#[cfg(feature = "mysql")]
pub struct MysqlConnection {
    pub client: Pool,
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

    #[inline(always)]
    pub async fn query<S, R>(
        stmt: S,
        params: &[&'_ dyn QueryParameter<'_>],
        conn: &MysqlConnection,
    ) -> Result<Vec<R>, Box<(dyn Error + Send + Sync)>>
    where
        S: AsRef<str> + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        Ok(execute_query(stmt, params, conn)
            .await?
            .iter()
            .flat_map(|row| R::deserialize_mysql(row))
            .collect())
    }

    #[inline(always)]
    pub(crate) async fn query_rows<'a>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &MysqlConnection,
    ) -> Result<CanyonRows, Box<(dyn Error + Send + Sync)>> {
        Ok(CanyonRows::MySQL(execute_query(stmt, params, conn).await?))
    }

    #[inline(always)]
    pub(crate) async fn query_one<'a, R>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &MysqlConnection,
    ) -> Result<Option<R::Output>, Box<(dyn Error + Send + Sync)>>
    where
        R: RowMapper,
    {
        let result = execute_query(stmt, params, conn).await?;

        match result.first() {
            Some(r) => Ok(Some(R::deserialize_mysql(r)?)),
            None => Ok(None),
        }
    }

    #[inline(always)]
    pub(crate) async fn query_one_for<'a, T: FromSqlOwnedValue<T>>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &MysqlConnection,
    ) -> Result<T, Box<(dyn Error + Send + Sync)>> {
        Ok(execute_query(stmt, params, conn)
            .await?
            .first()
            .ok_or_else(|| format!("Failure executing 'query_one_for' while retrieving the first row with stmt: {:?}", stmt))?
            .get::<T, usize>(0)
            .ok_or_else(|| format!("Failure executing 'query_one_for' while retrieving the first column value on the first row with stmt: {:?}", stmt))?
        )
    }

    #[inline(always)]
    async fn execute_query<S>(
        stmt: S,
        params: &[&'_ dyn QueryParameter<'_>],
        conn: &MysqlConnection,
    ) -> Result<Vec<Row>, Box<(dyn Error + Send + Sync)>>
    where
        S: AsRef<str> + Send,
    {
        let mysql_connection = conn.client.get_conn().await?;
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

    pub(crate) async fn execute<S>(
        stmt: S,
        params: &[&'_ dyn QueryParameter<'_>],
        conn: &MysqlConnection,
    ) -> Result<u64, Box<(dyn Error + Send + Sync)>>
    where
        S: AsRef<str> + Send,
    {
        let mysql_connection = conn.client.get_conn().await?;
        let mysql_stmt = generate_mysql_stmt(stmt.as_ref(), params)?;

        Ok(mysql_stmt.run(mysql_connection).await?.affected_rows())
    }

    #[cfg(feature = "mysql")]
    fn generate_mysql_stmt(
        stmt: &str,
        params: &[&'_ dyn QueryParameter<'_>],
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

    #[cfg(feature = "mysql")]
    fn reorder_params<T>(
        stmt: &str,
        params: &[&'_ dyn QueryParameter<'_>],
        fn_parser: impl Fn(&&dyn QueryParameter<'_>) -> T,
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
