#![allow(unused_imports)]

use crate::constants;
use canyon_sql::connection::DbConnection;
use canyon_sql::core::Canyon;
/// Integration tests for the migrations feature of `Canyon-SQL`
use canyon_sql::core::Transaction;
use canyon_sql::migrations::handler::Migrations;

/// Brings the information of the `PostgreSQL` requested schema
#[cfg(all(feature = "postgres", feature = "migrations"))]
#[canyon_sql::macros::canyon_tokio_test]
fn test_migrations_postgresql_status_query() {
    let canyon = Canyon::instance().unwrap();

    let ds = canyon.find_datasource_by_name_or_default(constants::PSQL_DS);
    assert!(ds.is_ok());
    let ds = ds.unwrap();
    let ds_name = &ds.name;

    let db_conn = canyon.get_connection(ds_name).await.unwrap_or_else(|_| {
        panic!(
            "Unable to get a database connection on Canyon Memory: {:?}",
            ds_name
        )
    });

    let results = db_conn
        .query_rows(constants::FETCH_PUBLIC_SCHEMA, &[])
        .await;
    assert!(results.is_ok());

    let res = results.unwrap();
    let public_schema_info = res.get_postgres_rows();
    let first_result = public_schema_info.first().unwrap();

    assert_eq!(first_result.columns().first().unwrap().name(), "table_name");
    assert_eq!(
        first_result.columns().first().unwrap().type_().name(),
        "name"
    );
    assert_eq!(first_result.columns().first().unwrap().type_().oid(), 19);
    assert_eq!(
        first_result.columns().first().unwrap().type_().schema(),
        "pg_catalog"
    );
}
