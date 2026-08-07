use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_insert_entity_function_tokens(table_schema_data: &str) -> syn::Result<TokenStream> {
    let insert_entity_signature = __detail::generate_insert_entity_signature();

    let insert_entity_with_signature = __detail::generate_insert_entity_with_signature();

    let insert_entity_body = __detail::generate_insert_entity_body(table_schema_data);

    let insert_entity_with_body = __detail::generate_insert_entity_with_body(table_schema_data);

    Ok(quote! {
        #insert_entity_signature {
            #insert_entity_body
        }

        #insert_entity_with_signature {
            #insert_entity_with_body
        }
    })
}

mod __detail {
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::query_operations::consts;

    pub(crate) fn generate_insert_entity_body(table_schema_data: &str) -> TokenStream {
        let default_db_conn_and_type = consts::generate_default_db_conn_and_type_tokens();

        let insert_execution =
            generate_insert_execution(table_schema_data, quote! { default_db_conn });

        quote! {
            #default_db_conn_and_type
            #insert_execution

            Ok(())
        }
    }

    pub(crate) fn generate_insert_entity_with_body(table_schema_data: &str) -> TokenStream {
        let insert_execution = generate_insert_execution(table_schema_data, quote! { input });

        quote! {
            let db_type = input.get_database_type()?;

            #insert_execution

            Ok(())
        }
    }

    fn generate_insert_execution(table_schema_data: &str, connection: TokenStream) -> TokenStream {
        let no_fields_to_insert_err =
            crate::query_operations::insert::__shared::no_fields_to_insert_err();

        quote! {
            use canyon_sql::connection::DbConnection;
            use canyon_sql::query::querybuilder::{
                InsertQueryBuilderOps,
                QueryBuilderOps,
            };

            let columns =
                <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>
                    ::field_columns();

            if columns.is_empty() {
                return #no_fields_to_insert_err;
            }

            let values =
                <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>
                    ::field_values(entity);

            let statement =
                canyon_sql::query::querybuilder::InsertQueryBuilder::new(
                    #table_schema_data,
                    db_type,
                )
                .with_known_columns(columns);

            if let Some(primary_key_column) =
                <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>
                    ::primary_key_column()
            {
                let statement = statement
                    .returning_columns(
                        ::core::iter::once(primary_key_column),
                    )
                    .build()?;

                let primary_key = #connection
                    .query_one_for::<
                        <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>
                            ::PrimaryKey
                    >(
                        statement.sql(),
                        &values,
                    )
                    .await?;

                <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>
                    ::set_primary_key(entity, primary_key)?;
            } else {
                let statement = statement.build()?;

                #connection
                    .execute(statement.sql(), &values)
                    .await?;
            }
        }
    }

    pub(crate) fn generate_insert_entity_signature() -> TokenStream {
        quote! {
            async fn insert_entity<'canyon_lt, 'err_lt, Entity>(
                entity: &'canyon_lt mut Entity,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
            where
                Entity: canyon_sql::core::RowMapper
                    + canyon_sql::query::bounds::EntityRuntimeInfo
                    + Sync
                    + 'canyon_lt
        }
    }

    pub(crate) fn generate_insert_entity_with_signature() -> TokenStream {
        quote! {
            async fn insert_entity_with<'canyon_lt, 'err_lt, Entity, Input>(
                entity: &'canyon_lt mut Entity,
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
