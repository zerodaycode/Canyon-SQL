use crate::query_operations::consts;
use proc_macro2::TokenStream;
use quote::quote;
use std::borrow::Cow;

pub fn generate_count_operations_tokens(table_schema_data: &str) -> TokenStream {
    let table_metadata =
        canyon_core::query::querybuilder::syntax::table_metadata::TableMetadata::from(
            table_schema_data,
        );
    let schema_name = table_metadata.schema;
    let table_name = table_metadata.name;
    let count = create_count_macro(schema_name.clone(), table_name.as_ref());
    let count_with = create_count_with_macro(schema_name, table_name.as_ref());

    quote! {
        #count
        #count_with
    }
}

pub fn create_count_macro(schema_name: Option<Cow<str>>, table_name: &str) -> TokenStream {
    let mssql_arm = get_mssql_arm_tokens_if_enabled(false);
    let schema_tokens = get_schema_tokens(schema_name);
    let table_name = create_cow_borrowed_table_name(table_name);
    let default_db_conn_and_type_tokens = consts::generate_default_db_conn_and_type_tokens();

    quote! {
        async fn count() -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
            use canyon_sql::connection::DbConnection;
            use canyon_sql::query::querybuilder::{QueryBuilderOps, SelectQueryBuilderOps};

            #default_db_conn_and_type_tokens

            let query = canyon_sql::query::querybuilder::SelectQueryBuilder::new_from_parts(
                #schema_tokens,
                #table_name,
                db_type
            ).count()
            .build()?;

            match db_type {
                #mssql_arm
                _ => {
                     default_db_conn.query_one_for::<i64>(query.sql(), query.params()).await
                }
            }
        }
    }
}

pub fn create_count_with_macro(schema_name: Option<Cow<str>>, table_name: &str) -> TokenStream {
    let mssql_arm = get_mssql_arm_tokens_if_enabled(true);
    let schema_tokens = get_schema_tokens(schema_name);
    let table_name = create_cow_borrowed_table_name(table_name);

    quote! {
        async fn count_with<'a, I>(input: I)
            -> Result<i64, Box<dyn std::error::Error + Send + Sync + 'a>>
        where
            I: canyon_sql::connection::DbConnection + Send + 'a
        {
            use canyon_sql::connection::DbConnection;
            use canyon_sql::query::querybuilder::{QueryBuilderOps, SelectQueryBuilderOps};

            let db_type = input.get_database_type()?;
            let query = canyon_sql::query::querybuilder::SelectQueryBuilder::new_from_parts(
                #schema_tokens,
                #table_name,
                db_type)
            .count()
            .build()?;

            match db_type {
                #mssql_arm
                _ => {
                    input.query_one_for::<i64>(query.sql(), query.params()).await
                }
            }
        }
    }
}

fn get_mssql_arm_tokens_if_enabled(is_with_input: bool) -> TokenStream {
    if !cfg!(feature = "mssql") {
        return quote! {};
    }

    let base_expr = quote! {
        let count_i32: i32 =
    };

    let query_call = if is_with_input {
        quote! {
            input.query_one_for::<i32>(query.sql(), query.params()).await?;
        }
    } else {
        quote! {
            default_db_conn.query_one_for::<i32>(query.sql(), query.params()).await?;
        }
    };

    quote! {
        canyon_sql::connection::DatabaseType::SqlServer => {
            #base_expr #query_call
            Ok(count_i32 as i64)
        }
    }
}

fn get_schema_tokens(schema_name: Option<Cow<'_, str>>) -> TokenStream {
    match schema_name {
        Some(schema_name) => quote! { Some(#schema_name) },
        None => quote! { None },
    }
}

fn create_cow_borrowed_table_name(table_name: &str) -> TokenStream {
    quote! { std::borrow::Cow::Borrowed(#table_name) }
}
