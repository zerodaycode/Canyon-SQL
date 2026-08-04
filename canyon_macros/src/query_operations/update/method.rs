use crate::query_operations::update::__err;
use crate::utils::helpers;
use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn generate_update_method_tokens(
    macro_data: &MacroTokens,
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    let mut update_ops_tokens = TokenStream::new();

    if let Some(primary_key) = macro_data.get_primary_key_field_annotation() {
        let update_columns = helpers::get_struct_fields_as_table_column_pairs_pk_parsed(macro_data);
        let update_values = __details::generate_update_values(macro_data, primary_key.ident);

        let query =
            __details::generate_update_stmt(table_schema_data, update_columns, &primary_key.name);

        let update_method_tokens =
            __details::generate_update_method_tokens(macro_data, &query, &update_values);
        let update_with_method_tokens =
            __details::generate_update_with_method_tokens(&query, &update_values);

        update_ops_tokens.extend(quote! {
            #update_method_tokens
            #update_with_method_tokens
        });
    } else {
        // If there's no primary key, update method over self won't be available.
        // Use instead the update associated function of the querybuilder
        __details::handle_no_primary_key_case(&mut update_ops_tokens);
    }

    Ok(update_ops_tokens)
}

mod __details {
    use super::*;
    use crate::query_operations::consts;
    use proc_macro2::Ident;

    pub(crate) fn generate_update_method_tokens(
        macro_data: &MacroTokens,
        query: &TokenStream,
        update_values: &Vec<TokenStream>,
    ) -> TokenStream {
        let ty = macro_data.ty;
        let (_, ty_generics, _) = macro_data.generics.split_for_impl();

        let update_signature = __signatures::get_update_signature();
        let default_db_conn_and_type_tokens = consts::generate_default_db_conn_and_type_tokens();

        quote! {
            #update_signature {
                use canyon_sql::query::querybuilder::{QueryBuilderOps, UpdateQueryBuilderOps};

                #default_db_conn_and_type_tokens

                let query = #query;
                let update_values: &[&dyn canyon_sql::query::QueryParameter] = &[#(#update_values),*];
                <#ty #ty_generics as canyon_sql::core::Transaction>::execute(query.as_ref(), update_values, default_db_conn).await
            }
        }
    }

    pub(crate) fn generate_update_with_method_tokens(
        query: &TokenStream,
        update_values: &Vec<TokenStream>,
    ) -> TokenStream {
        let update_with_signature = __signatures::get_update_with_signature();

        quote! {
            #update_with_signature {
                use canyon_sql::query::querybuilder::{QueryBuilderOps, UpdateQueryBuilderOps};
                let db_type = input.get_database_type()?;
                let query = #query;
                let update_values: &[&dyn canyon_sql::query::QueryParameter] = &[#(#update_values),*];
                input.execute(query.as_ref(), update_values).await
            }
        }
    }

    pub(crate) fn generate_update_stmt(
        table_schema_data: &str,
        update_columns: Vec<TokenStream>,
        pk_name: &str,
    ) -> TokenStream {
        quote! {
            canyon_sql::query::querybuilder::UpdateQueryBuilder::new_for(
                #table_schema_data, // TODO: construct a const value
                db_type,
            )
                .set(vec![#(#update_columns),*])?
                .r#where(
                    #pk_name,
                    canyon_sql::query::operators::Operator::Eq,
                )
                .build()?;
        }
    }

    pub(crate) fn generate_update_values(macro_data: &MacroTokens, pk: &Ident) -> Vec<TokenStream> {
        macro_data
            .get_fields_idents_skipping_pk()
            .map(|ident| {
                quote! {
                    &self.#ident as &dyn canyon_sql::query::QueryParameter
                }
            })
            .chain(std::iter::once(quote! {
                &self.#pk as &dyn canyon_sql::query::QueryParameter
            }))
            .collect::<Vec<_>>()
    }

    pub(crate) fn handle_no_primary_key_case(update_ops_tokens: &mut TokenStream) {
        let update_signature = __signatures::get_update_signature();
        let update_with_signature = __signatures::get_update_with_signature();

        let no_pk_err = __err::generate_no_pk_err();

        update_ops_tokens.extend(quote! {
            #update_signature { #no_pk_err }
            #update_with_signature{ #no_pk_err }
        });
    }
}

mod __signatures {
    use proc_macro2::TokenStream;
    use quote::quote;

    pub(crate) fn get_update_signature() -> TokenStream {
        quote! {
            async fn update(&self) -> Result<u64, Box<dyn std::error::Error + Sync + std::marker::Send>>
        }
    }

    pub(crate) fn get_update_with_signature() -> TokenStream {
        quote! {
            async fn update_with<'a, I>(&self, input: I)
                -> Result<u64, Box<dyn std::error::Error + Sync + std::marker::Send + 'a>>
            where I: canyon_sql::connection::DbConnection + Send + 'a
        }
    }
}
