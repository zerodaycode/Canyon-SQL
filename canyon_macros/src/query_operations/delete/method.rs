use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_delete_method_tokens(
    macro_data: &MacroTokens,
    table_schema_data: &str,
) -> TokenStream {
    let mut delete_ops_tokens = TokenStream::new();

    let ty = macro_data.ty;
    let (_, ty_generics, _) = macro_data.generics.split_for_impl();
    let pk = macro_data.get_primary_key_field_annotation();

    let delete_with_signature = __signatures::get_delete_with_signature();

    if let Some(primary_key) = pk {
        let query = __detail::generate_delete_stmt(table_schema_data, primary_key);
        let pk_field_value = __detail::get_pk_field_value(&primary_key.ident);

        let delete_method_tokens =
            __detail::generate_delete_method_tokens(macro_data, &query, &pk_field_value);
        let delete_with_method_tokens =
            __detail::generate_delete_with_method_tokens(&query, pk_field_value);

        delete_ops_tokens.extend(quote! {
            #delete_method_tokens
            #delete_with_method_tokens
        });
    } else {
        __detail::handle_no_primary_key_case(&mut delete_ops_tokens);
    }

    delete_ops_tokens
}

mod __detail {
    use crate::{
        query_operations::delete::{__err::generate_no_pk_err, method::__signatures},
        utils::{macro_tokens::MacroTokens, primary_key_attribute::PrimaryKeyAttribute},
    };
    use proc_macro2::{Ident, TokenStream};
    use quote::quote;

    pub(crate) fn generate_delete_stmt(
        table_schema_data: &str,
        primary_key_attribute: &PrimaryKeyAttribute,
    ) -> TokenStream {
        let pk_name = &primary_key_attribute.name;
        quote! {
            canyon_sql::query::querybuilder::DeleteQueryBuilder::new_for(
                #table_schema_data, // TODO: construct a const value
                db_type,
            )
                .r#where(
                    #pk_name,
                    canyon_sql::query::operators::Operator::Eq,
                )
                .build()?;
        }
    }

    pub(crate) fn get_pk_field_value(pk_field: &Ident) -> TokenStream {
        quote! { &self.#pk_field as &dyn canyon_sql::query::QueryParameter }
    }

    pub(crate) fn generate_delete_method_tokens(
        macro_tokens: &MacroTokens,
        query: &TokenStream,
        pk_field_value: &TokenStream,
    ) -> TokenStream {
        let ty = macro_tokens.ty;
        let (_, ty_generics, _) = macro_tokens.generics.split_for_impl();

        let delete_signature = __signatures::get_delete_signature();

        quote! {
            #delete_signature {
                use canyon_sql::query::querybuilder::{QueryBuilderOps, DeleteQueryBuilderOps};

                let default_conn = canyon_sql::core::Canyon::instance()?.get_default_connection()?;
                let db_type = default_conn.get_database_type()?;

                let query = #query;
                <#ty #ty_generics as canyon_sql::core::Transaction>::execute(query.as_ref(), &[#pk_field_value], default_conn).await?;
                Ok(())
            }
        }
    }

    pub(crate) fn generate_delete_with_method_tokens(
        query: &TokenStream,
        pk_field_value: TokenStream,
    ) -> TokenStream {
        let delete_with_signature = __signatures::get_delete_with_signature();

        quote! {
            #delete_with_signature {
                use canyon_sql::query::querybuilder::{QueryBuilderOps, DeleteQueryBuilderOps};

                let db_type = input.get_database_type()?;
                let query = #query;
                input.execute(query.as_ref(), &[#pk_field_value]).await?;
                Ok(())
            }
        }
    }

    // Delete operation over an instance isn't available without declaring a primary key.
    // The delete querybuilder variant must be used for the case when there's no pk declared
    pub(crate) fn handle_no_primary_key_case(delete_ops_tokens: &mut TokenStream) {
        let delete_signature = __signatures::get_delete_signature();
        let delete_with_signature = __signatures::get_delete_with_signature();

        let no_pk_error = generate_no_pk_err();

        delete_ops_tokens.extend(quote! {
            #delete_signature { #no_pk_error }
            #delete_with_signature { #no_pk_error }
        });
    }
}

mod __signatures {
    use proc_macro2::TokenStream;
    use quote::quote;

    pub(crate) fn get_delete_signature() -> TokenStream {
        quote! {
            async fn delete(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        }
    }

    pub(crate) fn get_delete_with_signature() -> TokenStream {
        quote! {
            async fn delete_with<'canyon, 'err, I>(&self, input: I) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'err)>>
                where I: canyon_sql::connection::DbConnection + Send + 'canyon
        }
    }
}
