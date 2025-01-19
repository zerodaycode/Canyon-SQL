#![allow(unused_imports)]
use crate::constants;
/// Integration tests for the migrations feature of `Canyon-SQL`
use canyon_sql::core::Transaction;
#[cfg(feature = "migrations")]
use canyon_sql::migrations::handler::Migrations;

/// Brings the information of the `PostgreSQL` requested schema
#[cfg(all(feature = "postgres", feature = "migrations"))]
#[canyon_sql::macros::canyon_tokio_test]
fn test_migrations_postgresql_status_query() {
    let conn_res =
        canyon_sql::connection::get_database_connection_by_ds(Some(constants::PSQL_DS)).await;
    assert!(conn_res.is_ok());

    let db_conn = &mut conn_res.unwrap();
    let results = Migrations::query(constants::FETCH_PUBLIC_SCHEMA, [], db_conn).await;
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
