use crate::query_operations::consts;
use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};

pub fn generate_find_by_pk_operations_tokens(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &str,
) -> Result<TokenStream, Box<dyn std::error::Error + Send + Sync>> {
    let ty = macro_data.ty;
    let mapper_ty = macro_data.retrieve_mapping_target_type().as_ref();
    let pk = macro_data.get_primary_key_annotation();

    let base_body = if let Some(compile_time_known_pk) = pk {
        Some(quote! {
            let stmt = format!(
                "SELECT * FROM {} WHERE {} = $1",
                #table_schema_data, #compile_time_known_pk
            );
        })
    } else {
        let tt = mapper_ty.unwrap_or(ty).to_token_stream();
        Some(quote! {
            use canyon_sql::query::bounds::Inspectionable;
            let pk = <#tt as Inspectionable>::primary_key_st()
                .ok_or_else(|| "No primary key found for this instance")?;
            let stmt = format!(
                "SELECT * FROM {} WHERE {} = $1",
                #table_schema_data,
                pk
            );
        })
    };

    let mapper_ty = mapper_ty.unwrap_or(ty);
    let find_by_pk = create_find_by_pk_macro(mapper_ty, &base_body)?;
    let find_by_pk_with = create_find_by_pk_with(mapper_ty, &base_body)?;

    Ok(quote! {
        #find_by_pk
        #find_by_pk_with
    })
}

pub fn create_find_by_pk_macro(
    mapper_ty: &Ident,
    base_body: &Option<TokenStream>,
) -> Result<TokenStream, Box<dyn std::error::Error + Send + Sync>> {
    let body = if let Some(body) = base_body {
        let default_db_conn_call = consts::generate_default_db_conn_tokens();
        quote! {
            #body;
            #default_db_conn_call
                .query_one::<#mapper_ty>(&stmt, &[value])
                .await
        }
    } else {
        let unsupported_op_err = consts::generate_no_pk_error();
        quote! { #unsupported_op_err }
    };

    Ok(quote! {
        async fn find_by_pk<'canyon_lt, 'err_lt>(value: &'canyon_lt dyn canyon_sql::query::QueryParameter)
            -> Result<Option<#mapper_ty>, Box<(dyn std::error::Error + Send + Sync + 'err_lt)>>
        {
            #body
        }
    })
}

pub fn create_find_by_pk_with(
    mapper_ty: &Ident,
    base_body: &Option<TokenStream>,
) -> Result<TokenStream, Box<dyn std::error::Error + Send + Sync>> {
    let body = if let Some(body) = base_body {
        quote! {
            #body;
            input.query_one::<#mapper_ty>(&stmt, &[value]).await
        }
    } else {
        let unsupported_op_err = consts::generate_no_pk_error();
        quote! { #unsupported_op_err }
    };

    Ok(quote! {
        async fn find_by_pk_with<'canyon_lt, 'err_lt, I>(value: &'canyon_lt dyn canyon_sql::query::QueryParameter, input: I)
            -> Result<Option<#mapper_ty>, Box<(dyn std::error::Error + Send + Sync + 'err_lt)>>
        where
            I: canyon_sql::connection::DbConnection + Send + 'canyon_lt
        {
            #body
        }
    })
}
