use crate::query_operations::consts;
use crate::utils::{helpers, macro_tokens::MacroTokens};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate_find_by_pk_operations_tokens(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &str,
) -> Result<TokenStream, Box<dyn std::error::Error + Send + Sync>> {
    let ty = macro_data.ty;
    let mapping_target_ty = macro_data.retrieve_mapping_target_type().as_ref();

    let operations = match mapping_target_ty {
        Some(entity_ty) => generate_mapped_find_by_pk_operations(entity_ty, table_schema_data),
        None => {
            let Some(primary_key) = macro_data.get_primary_key_annotation() else {
                return Ok(generate_unsupported_find_by_pk_operations(ty));
            };

            let columns = helpers::get_struct_fields_as_column_ref_token_stream(macro_data, false);

            generate_entity_find_by_pk_operations(ty, table_schema_data, &columns, &primary_key)
        }
    };

    Ok(operations)
}

fn generate_entity_find_by_pk_operations(
    result_ty: &Ident,
    table_schema_data: &str,
    columns: &TokenStream,
    primary_key: &str,
) -> TokenStream {
    let query = quote! {
        let stmt =
            canyon_sql::query::querybuilder::SelectQueryBuilder::new(
                #table_schema_data,
                db_type,
            )
            .with_known_columns(#columns)
            .r#where(
                #primary_key,
                canyon_sql::query::operators::Operator::Eq,
            )
            .build()?;
    };

    generate_find_by_pk_methods(result_ty, query)
}

fn generate_mapped_find_by_pk_operations(
    entity_ty: &Ident,
    table_schema_data: &str,
) -> TokenStream {
    let query = quote! {
        use canyon_sql::query::bounds::EntityRuntimeInfo;

        let primary_key =
            <#entity_ty as EntityRuntimeInfo>::primary_key_name()
                .ok_or_else(|| "No primary key found for this entity")?;

        let stmt =
            canyon_sql::query::querybuilder::SelectQueryBuilder::new(
                #table_schema_data,
                db_type,
            )
            .r#where(
                primary_key,
                canyon_sql::query::operators::Operator::Eq,
            )
            .build()?;
    };

    generate_find_by_pk_methods(entity_ty, query)
}

fn generate_find_by_pk_methods(result_ty: &Ident, query: TokenStream) -> TokenStream {
    let find_by_pk = generate_find_by_pk(result_ty, &query);
    let find_by_pk_with = generate_find_by_pk_with(result_ty, &query);

    quote! {
        #find_by_pk
        #find_by_pk_with
    }
}

fn generate_find_by_pk(result_ty: &Ident, query: &TokenStream) -> TokenStream {
    let default_db_conn_call = consts::generate_default_db_conn_tokens();

    quote! {
        async fn find_by_pk<'canyon_lt, 'err_lt>(
            value: &'canyon_lt dyn canyon_sql::query::QueryParameter,
        ) -> Result<
            Option<#result_ty>,
            Box<dyn std::error::Error + Send + Sync + 'err_lt>,
        > {
            use canyon_sql::query::querybuilder::{
                QueryBuilderOps,
                SelectQueryBuilderOps,
            };

            let input = {
                #default_db_conn_call
            };

            let db_type = input.get_database_type()?;

            #query

            input
                .query_one::<#result_ty>(stmt.as_ref(), &[value])
                .await
        }
    }
}

fn generate_find_by_pk_with(result_ty: &Ident, query: &TokenStream) -> TokenStream {
    quote! {
        async fn find_by_pk_with<'canyon_lt, 'err_lt, I>(
            value: &'canyon_lt dyn canyon_sql::query::QueryParameter,
            input: I,
        ) -> Result<
            Option<#result_ty>,
            Box<dyn std::error::Error + Send + Sync + 'err_lt>,
        >
        where
            I: canyon_sql::connection::DbConnection
                + Send
                + 'canyon_lt,
        {
            use canyon_sql::query::querybuilder::{
                QueryBuilderOps,
                SelectQueryBuilderOps,
            };

            let db_type = input.get_database_type()?;

            #query

            input
                .query_one::<#result_ty>(stmt.as_ref(), &[value])
                .await
        }
    }
}

fn generate_unsupported_find_by_pk_operations(result_ty: &Ident) -> TokenStream {
    let find_by_pk_error = consts::generate_no_pk_error();
    let find_by_pk_with_error = consts::generate_no_pk_error();

    quote! {
        async fn find_by_pk<'canyon_lt, 'err_lt>(
            _value: &'canyon_lt dyn canyon_sql::query::QueryParameter,
        ) -> Result<
            Option<#result_ty>,
            Box<dyn std::error::Error + Send + Sync + 'err_lt>,
        > {
            #find_by_pk_error
        }

        async fn find_by_pk_with<'canyon_lt, 'err_lt, I>(
            _value: &'canyon_lt dyn canyon_sql::query::QueryParameter,
            _input: I,
        ) -> Result<
            Option<#result_ty>,
            Box<dyn std::error::Error + Send + Sync + 'err_lt>,
        >
        where
            I: canyon_sql::connection::DbConnection
                + Send
                + 'canyon_lt,
        {
            #find_by_pk_with_error
        }
    }
}
