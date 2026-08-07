use crate::query_operations::consts;
use crate::utils::helpers;
use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate_find_all_operations_tokens(
    mapper_ty: &Ident,
    table_schema_data: &str,
    macro_data: &MacroTokens,
) -> TokenStream {
    let columns = helpers::get_struct_fields_as_column_ref_token_stream(macro_data, false);
    let find_all = create_find_all_macro(mapper_ty, table_schema_data, &columns);
    let find_all_with = create_find_all_with_macro(mapper_ty, table_schema_data, &columns);

    quote! {
        #find_all
        #find_all_with
    }
}

fn create_find_all_macro(
    mapper_ty: &Ident,
    table_schema_data: &str,
    columns: &TokenStream,
) -> TokenStream {
    let default_db_conn_and_type_tokens = consts::generate_default_db_conn_and_type_tokens();

    quote! {
        async fn find_all()
            -> Result<Vec<#mapper_ty>, Box<(dyn std::error::Error + Send + Sync)>>
        {
            use canyon_sql::connection::DbConnection;
            use crate::canyon_sql::query::querybuilder::SelectQueryBuilderOps;

            #default_db_conn_and_type_tokens
            let stmt = canyon_sql::query::querybuilder::SelectQueryBuilder::new(#table_schema_data, db_type)
                .with_known_columns(#columns)
                .build()?;
            default_db_conn.query(stmt.sql(), &[]).await
        }
    }
}

fn create_find_all_with_macro(
    mapper_ty: &Ident,
    table_schema_data: &str,
    columns: &TokenStream,
) -> TokenStream {
    quote! {
        async fn find_all_with<'a, I>(input: I)
            -> Result<Vec<#mapper_ty>, Box<(dyn std::error::Error + Send + Sync)>>
        where
            I: canyon_sql::connection::DbConnection + Send + 'a
        {
            use canyon_sql::connection::DbConnection;
            use canyon_sql::crud::ReadOperations;
            use crate::canyon_sql::query::querybuilder::SelectQueryBuilderOps;

            let db_type = input.get_database_type()?;
            let stmt = canyon_sql::query::querybuilder::SelectQueryBuilder::new(#table_schema_data, db_type)
                .with_known_columns(#columns)
                .build()?;
            input.query::<&str, #mapper_ty>(stmt.sql(), &[]).await
        }
    }
}
