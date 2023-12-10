#![allow(unused_imports)]
use crate::constants;
/// Integration tests for the migrations feature of `Canyon-SQL`
use canyon_sql::crud::Transaction;
#[cfg(feature = "migrations")]
use canyon_sql::migrations::handler::Migrations;

/// Brings the information of the `PostgreSQL` requested schema
#[cfg(all(feature = "postgres", feature = "migrations"))]
#[canyon_sql::macros::canyon_tokio_test]
fn test_migrations_postgresql_status_query() {
    let results = Migrations::query(constants::FETCH_PUBLIC_SCHEMA, [], constants::PSQL_DS).await;
    assert!(results.is_ok());

    let res = results.unwrap();
    let public_schema_info = res.get_postgres_rows();
    let first_result = public_schema_info.get(0).unwrap();

    assert_eq!(first_result.columns().get(0).unwrap().name(), "table_name");
    assert_eq!(
        first_result.columns().get(0).unwrap().type_().name(),
        "name"
    );
    assert_eq!(first_result.columns().get(0).unwrap().type_().oid(), 19);
    assert_eq!(
        first_result.columns().get(0).unwrap().type_().schema(),
        "pg_catalog"
    );
}
