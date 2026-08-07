use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn generate_update_entity_tokens(table_schema_data: &str) -> syn::Result<TokenStream> {
    let update_entity_signature = __detail::generate_update_entity_signature();

    let update_entity_with_signature = __detail::generate_update_entity_with_signature();

    let update_entity_body = __detail::generate_update_entity_body(table_schema_data);

    let update_entity_with_body = __detail::generate_update_entity_with_body(table_schema_data);

    Ok(quote! {
        #update_entity_signature {
            #update_entity_body
        }

        #update_entity_with_signature {
            #update_entity_with_body
        }
    })
}

mod __detail {
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::query_operations::consts;

    pub(crate) fn generate_update_entity_body(table_schema_data: &str) -> TokenStream {
        let default_db_conn_and_type = consts::generate_default_db_conn_and_type_tokens();

        let update_execution =
            generate_update_execution(table_schema_data, quote! { default_db_conn });

        quote! {
            #default_db_conn_and_type
            #update_execution

            Ok(())
        }
    }

    pub(crate) fn generate_update_entity_with_body(table_schema_data: &str) -> TokenStream {
        let update_execution = generate_update_execution(table_schema_data, quote! { input });

        quote! {
            let db_type = input.get_database_type()?;

            #update_execution

            Ok(())
        }
    }

    fn generate_update_execution(table_schema_data: &str, connection: TokenStream) -> TokenStream {
        quote! {
            use canyon_sql::connection::DbConnection;
            use canyon_sql::query::querybuilder::{
                QueryBuilderOps,
                UpdateQueryBuilderOps,
            };

            let primary_key_name =
                <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>
                    ::primary_key_name()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Cannot update an entity without a primary key",
                        )
                    })?;

            let primary_key_value =
                <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>
                    ::primary_key_value(entity)
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Cannot update an entity without a primary-key value",
                        )
                    })?;

            let update_columns =
                <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>
                    ::field_columns();

            let mut update_values =
                <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>
                    ::field_values(entity);

            update_values.push(primary_key_value);

            let query =
                canyon_sql::query::querybuilder::UpdateQueryBuilder::new(
                    #table_schema_data,
                    db_type,
                )
                .set(update_columns)?
                .r#where(
                    primary_key_name,
                    canyon_sql::query::operators::Operator::Eq,
                )
                .build()?;

            #connection
                .execute(
                    query.as_ref(),
                    &update_values,
                )
                .await?;
        }
    }

    pub(crate) fn generate_update_entity_signature() -> TokenStream {
        quote! {
            async fn update_entity<'canyon_lt, 'err_lt, Entity>(
                entity: &'canyon_lt Entity,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
            where
                Entity: canyon_sql::core::RowMapper
                    + canyon_sql::query::bounds::EntityRuntimeInfo
                    + Sync
                    + 'canyon_lt
        }
    }

    pub(crate) fn generate_update_entity_with_signature() -> TokenStream {
        quote! {
            async fn update_entity_with<'canyon_lt, 'err_lt, Entity, Input>(
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
