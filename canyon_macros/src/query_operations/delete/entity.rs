use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn generate_delete_entity_tokens(table_schema_data: &str) -> syn::Result<TokenStream> {
    let delete_entity_signature = __detail::generate_delete_entity_signature();
    let delete_entity_with_signature = __detail::generate_delete_entity_with_signature();

    let delete_entity_body = __detail::generate_delete_entity_body(table_schema_data);
    let delete_entity_with_body = __detail::generate_delete_entity_with_body(table_schema_data);

    Ok(quote! {
        #delete_entity_signature {
            #delete_entity_body
        }

        #delete_entity_with_signature {
            #delete_entity_with_body
        }
    })
}

mod __detail {
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::query_operations::consts;

    pub(crate) fn generate_delete_entity_body(table_schema_data: &str) -> TokenStream {
        let default_db_conn_and_type = consts::generate_default_db_conn_and_type_tokens();

        let delete_execution =
            generate_delete_execution(table_schema_data, quote! { default_db_conn });

        quote! {
            #default_db_conn_and_type
            #delete_execution

            Ok(())
        }
    }

    pub(crate) fn generate_delete_entity_with_body(table_schema_data: &str) -> TokenStream {
        let delete_execution = generate_delete_execution(table_schema_data, quote! { input });

        quote! {
            let db_type = input.get_database_type()?;

            #delete_execution

            Ok(())
        }
    }

    fn generate_delete_execution(table_schema_data: &str, connection: TokenStream) -> TokenStream {
        quote! {
            use canyon_sql::query::querybuilder::{
                DeleteQueryBuilderOps,
                QueryBuilderOps,
            };

            let primary_key_name =
                match <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>::primary_key_name() {
                    Some(primary_key_name) => primary_key_name,
                    None => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Cannot delete an entity without a primary key",
                        )
                        .into());
                    }
                };

            let primary_key_value = match entity.primary_key_value() {
                Some(primary_key_value) => primary_key_value,
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Cannot delete an entity without a primary-key value",
                    )
                    .into());
                }
            };

            let query =
                canyon_sql::query::querybuilder::DeleteQueryBuilder::new_for(
                    #table_schema_data,
                    db_type,
                )
                .r#where(
                    primary_key_name,
                    canyon_sql::query::operators::Operator::Eq,
                )
                .build()?;

            #connection
                .execute(query.as_ref(), &[primary_key_value])
                .await?;
        }
    }

    pub(crate) fn generate_delete_entity_signature() -> TokenStream {
        quote! {
            async fn delete_entity<'canyon_lt, 'err_lt, Entity>(
                entity: &'canyon_lt Entity,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
            where
                Entity: canyon_sql::core::RowMapper
                    + canyon_sql::query::bounds::EntityRuntimeInfo
                    + Sync
                    + 'canyon_lt
        }
    }

    pub(crate) fn generate_delete_entity_with_signature() -> TokenStream {
        quote! {
            async fn delete_entity_with<'canyon_lt, 'err_lt, Entity, Input>(
                entity: &'canyon_lt Entity,
                input: Input,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
            where
                Entity: canyon_sql::core::RowMapper
                    + canyon_sql::query::bounds::EntityRuntimeInfo
                    + Sync
                    + 'canyon_lt,
                Input: canyon_sql::connection::DbConnection
                    + Send
                    + 'canyon_lt
        }
    }
}
