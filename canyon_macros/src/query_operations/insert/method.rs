use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::TokenStream;
use quote::quote;

// Generates the TokenStream for the _insert operation
pub(crate) fn generate_insert_method_tokens(
    macro_data: &MacroTokens,
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    let insert_signature = quote! {
        async fn insert<'a>(&mut self)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'a>>
    };
    let insert_with_signature = quote! {
        async fn insert_with<'a, I>(&mut self, input: I)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'a>>
        where
            I: canyon_sql::connection::DbConnection + Send + 'a
    };

    let insert_body;
    let insert_with_body;
    let insert_values;

    if macro_data.retrieve_mapping_target_type().is_some() {
        let raised_err = __details::generate_unsupported_operation_err();
        insert_body = raised_err.clone(); // TODO: Can't we do it better?
        insert_with_body = raised_err;
        insert_values = quote! {};
    } else {
        insert_values = __details::generate_insert_fn_values_slice_expr(macro_data);
        insert_body =
            __details::generate_insert_fn_body_tokens(macro_data, table_schema_data, false);
        insert_with_body =
            __details::generate_insert_fn_body_tokens(macro_data, table_schema_data, true);
    };

    Ok(quote! {
        #insert_signature {
            #insert_values
            #insert_body
        }

        #insert_with_signature {
            #insert_values
            #insert_with_body
        }
    })
}

mod __details {
    use super::*;
    use crate::utils::helpers;

    pub(crate) fn generate_insert_fn_body_tokens(
        macro_data: &MacroTokens,
        table_schema_data: &str,
        is_with_method: bool,
    ) -> TokenStream {
        let pk_ident_and_type = macro_data.get_primary_key_ident_and_type();
        let insert_columns =
            helpers::get_struct_fields_as_column_ref_token_stream(macro_data, true);

        let connection_initializer = if is_with_method {
            quote! { input }
        } else {
            quote! {
                canyon_sql::core::Canyon::instance()?
                    .get_default_connection()?
            }
        };

        let mut insert_body_tokens = TokenStream::new();
        insert_body_tokens.extend(quote! {
            use canyon_sql::connection::DbConnection;
            use canyon_sql::query::querybuilder::{InsertQueryBuilderOps, QueryBuilderOps};

            let db_conn = #connection_initializer;
            let insert_columns = #insert_columns;
            let stmt = canyon_sql::query::querybuilder::InsertQueryBuilder::new(
                #table_schema_data,
                db_conn.get_database_type()?,
            )
            .with_known_columns(insert_columns)
        });

        if let Some((pk_ident, pk_type)) = pk_ident_and_type.as_ref() {
            let primary_key = macro_data
                .get_primary_key_annotation()
                .expect("Primary key annotation must exist when primary key ident and type exist");

            let returning_columns = helpers::get_fields_as_iterable_of_column_refs(vec![(
                pk_ident.to_string(),
                primary_key,
            )]);

            insert_body_tokens.extend(quote! {
                .returning_columns(#returning_columns)
                .build()?;

                self.#pk_ident = db_conn
                    .query_one_for::<#pk_type>(stmt.sql(), values)
                    .await?;

                Ok(())
            });
        } else {
            insert_body_tokens.extend(quote! {
                .build()?;

                let _ = db_conn.execute(stmt.sql(), values).await?;

                Ok(())
            });
        }

        insert_body_tokens
    }

    pub(crate) fn generate_insert_fn_values_slice_expr(macro_data: &MacroTokens) -> TokenStream {
        // Retrieves the fields of the Struct
        let fields = macro_data.get_columns_skipping_pk();

        let insert_values = fields.map(|field| {
            let field = field
                .ident
                .as_ref()
                .expect("Error converting a Field to its ident on the insert");
            quote! { &self.#field }
        });

        quote! {
            let values: &[&dyn canyon_sql::query::QueryParameter] = &[#(#insert_values),*];
        }
    }

    pub(crate) fn generate_unsupported_operation_err() -> TokenStream {
        quote! {
            Err(
                std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Can't use the 'Insert' family transactions as a method (that receives self as first parameter) \
                    if your T type in CrudOperations is NOT the same type that implements RowMapper. \
                    Consider to use instead the provided insert_entity or insert_entity_with functions."
                ).into_inner().unwrap()
            )
        }
    }
}
