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
    let result_ty = mapping_target_ty.unwrap_or(ty);

    let columns =
        helpers::get_struct_fields_as_column_ref_token_stream(macro_data, false);

    let primary_key_resolution = match mapping_target_ty {
        Some(entity_ty) => {
            generate_inspectionable_primary_key_resolution(entity_ty)
        }
        None => {
            let Some(primary_key) = macro_data.get_primary_key_annotation() else {
                return Ok(generate_unsupported_find_by_pk_operations(
                    result_ty,
                ));
            };

            generate_compile_time_primary_key_resolution(&primary_key)
        }
    };

    let find_by_pk = generate_find_by_pk(
        result_ty,
        table_schema_data,
        &columns,
        &primary_key_resolution,
    );

    let find_by_pk_with = generate_find_by_pk_with(
        result_ty,
        table_schema_data,
        &columns,
        &primary_key_resolution,
    );

    Ok(quote! {
        #find_by_pk
        #find_by_pk_with
    })
}

fn generate_compile_time_primary_key_resolution(
    primary_key: &str,
) -> TokenStream {
    quote! {
        let primary_key = #primary_key;
    }
}

fn generate_inspectionable_primary_key_resolution(
    entity_ty: &Ident,
) -> TokenStream {
    quote! {
        use canyon_sql::query::bounds::Inspectionable;

        let primary_key =
            <#entity_ty as Inspectionable>::primary_key_st()
                .ok_or_else(|| "No primary key found for this entity")?;
    }
}

fn generate_find_by_pk(
    result_ty: &Ident,
    table_schema_data: &str,
    columns: &TokenStream,
    primary_key_resolution: &TokenStream,
) -> TokenStream {
    let default_db_conn_call =
        consts::generate_default_db_conn_tokens();

    let query = generate_find_by_pk_query(
        quote! { default_db_conn },
        table_schema_data,
        columns,
        primary_key_resolution,
    );

    quote! {
        async fn find_by_pk<'canyon_lt, 'err_lt>(
            value: &'canyon_lt dyn canyon_sql::query::QueryParameter,
        ) -> Result<
            Option<#result_ty>,
            Box<dyn std::error::Error + Send + Sync + 'err_lt>,
        > {
            let default_db_conn = {
                #default_db_conn_call
            };

            #query

            default_db_conn
                .query_one::<#result_ty>(stmt.as_ref(), &[value])
                .await
        }
    }
}

fn generate_find_by_pk_with(
    result_ty: &Ident,
    table_schema_data: &str,
    columns: &TokenStream,
    primary_key_resolution: &TokenStream,
) -> TokenStream {
    let query = generate_find_by_pk_query(
        quote! { input },
        table_schema_data,
        columns,
        primary_key_resolution,
    );

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
            #query

            input
                .query_one::<#result_ty>(stmt.as_ref(), &[value])
                .await
        }
    }
}

fn generate_find_by_pk_query(
    connection: TokenStream,
    table_schema_data: &str,
    columns: &TokenStream,
    primary_key_resolution: &TokenStream,
) -> TokenStream {
    quote! {
        use canyon_sql::query::querybuilder::{
            QueryBuilderOps,
            SelectQueryBuilderOps,
        };

        #primary_key_resolution

        let db_type = #connection.get_database_type()?;

        let stmt =
            canyon_sql::query::querybuilder::SelectQueryBuilder::new(
                #table_schema_data,
                db_type,
            )
            .with_known_columns(#columns)
            .r#where(
                primary_key,
                canyon_sql::query::operators::Operator::Eq,
            )
            .build()?;
    }
}

fn generate_unsupported_find_by_pk_operations(
    result_ty: &Ident,
) -> TokenStream {
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