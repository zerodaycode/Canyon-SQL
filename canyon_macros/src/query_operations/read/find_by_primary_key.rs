use crate::{
    query_operations::consts,
    utils::{helpers, macro_tokens::MacroTokens},
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate_find_by_pk_operations_tokens(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    let ty = macro_data.ty;

    let mapping_target_ty = macro_data.retrieve_mapping_target_type().as_ref();

    match mapping_target_ty {
        Some(mapped_ty) => {
            let query = generate_mapped_find_by_pk_query(mapped_ty, table_schema_data);

            Ok(generate_find_by_pk_methods(mapped_ty, &query))
        }

        None => {
            let Some(primary_key) = macro_data.get_primary_key_annotation() else {
                return Ok(generate_unsupported_find_by_pk_operations(ty));
            };

            let columns = helpers::get_struct_fields_as_column_ref_token_stream(macro_data, false);

            let query = generate_static_find_by_pk_query(table_schema_data, &columns, &primary_key);

            Ok(generate_find_by_pk_methods(ty, &query))
        }
    }
}

fn generate_static_find_by_pk_query(
    table_schema_data: &str,
    columns: &TokenStream,
    primary_key: &str,
) -> TokenStream {
    quote! {
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
    }
}

fn generate_mapped_find_by_pk_query(mapped_ty: &Ident, table_schema_data: &str) -> TokenStream {
    quote! {
        let primary_key =
            <#mapped_ty as canyon_sql::query::bounds::EntityRuntimeInfo>
                ::primary_key_name()
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        concat!(
                            "Cannot find by primary key because mapped entity `",
                            stringify!(#mapped_ty),
                            "` has no primary key",
                        ),
                    )
                })?;

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
    }
}

fn generate_find_by_pk_methods(result_ty: &Ident, query: &TokenStream) -> TokenStream {
    let find_by_pk = generate_find_by_pk(result_ty, query);

    let find_by_pk_with = generate_find_by_pk_with(result_ty, query);

    quote! {
        #find_by_pk
        #find_by_pk_with
    }
}

fn generate_find_by_pk(result_ty: &Ident, query: &TokenStream) -> TokenStream {
    let signature = __detail::generate_find_by_pk_signature(result_ty);

    let default_db_conn_call = consts::generate_default_db_conn_tokens();

    let body = quote! {
        use canyon_sql::connection::DbConnection;
        use canyon_sql::query::querybuilder::{
            QueryBuilderOps,
            SelectQueryBuilderOps,
        };

        let input = {
            #default_db_conn_call
        };

        let db_type =
            input.get_database_type()?;

        #query

        input
            .query_one::<#result_ty>(
                stmt.as_ref(),
                &[value],
            )
            .await
    };

    __detail::generate_method(signature, body)
}

fn generate_find_by_pk_with(result_ty: &Ident, query: &TokenStream) -> TokenStream {
    let signature = __detail::generate_find_by_pk_with_signature(result_ty);

    let body = quote! {
        use canyon_sql::connection::DbConnection;
        use canyon_sql::query::querybuilder::{
            QueryBuilderOps,
            SelectQueryBuilderOps,
        };

        let db_type =
            input.get_database_type()?;

        #query

        input
            .query_one::<#result_ty>(
                stmt.as_ref(),
                &[value],
            )
            .await
    };

    __detail::generate_method(signature, body)
}

fn generate_unsupported_find_by_pk_operations(result_ty: &Ident) -> TokenStream {
    let find_by_pk_signature = __detail::generate_find_by_pk_signature(result_ty);

    let find_by_pk_with_signature = __detail::generate_find_by_pk_with_signature(result_ty);

    let find_by_pk =
        __detail::generate_method(find_by_pk_signature, consts::generate_no_pk_error());

    let find_by_pk_with =
        __detail::generate_method(find_by_pk_with_signature, consts::generate_no_pk_error());

    quote! {
        #find_by_pk
        #find_by_pk_with
    }
}

mod __detail {
    use proc_macro2::{Ident, TokenStream};
    use quote::quote;

    pub(super) fn generate_find_by_pk_signature(result_ty: &Ident) -> TokenStream {
        quote! {
            async fn find_by_pk<'canyon_lt, 'err_lt>(
                value: &'canyon_lt dyn canyon_sql::query::QueryParameter,
            ) -> Result<Option<#result_ty>, Box<dyn std::error::Error + Send + Sync + 'err_lt>>
        }
    }

    pub(super) fn generate_find_by_pk_with_signature(result_ty: &Ident) -> TokenStream {
        quote! {
            async fn find_by_pk_with<'canyon_lt, 'err_lt, Input>(
                value: &'canyon_lt dyn canyon_sql::query::QueryParameter,
                input: Input,
            ) -> Result<Option<#result_ty>, Box<dyn std::error::Error + Send + Sync + 'err_lt>>
            where
                Input: canyon_sql::connection::DbConnection
                    + Send
                    + 'canyon_lt
        }
    }

    pub(super) fn generate_method(signature: TokenStream, body: TokenStream) -> TokenStream {
        quote! {
            #signature {
                #body
            }
        }
    }
}
