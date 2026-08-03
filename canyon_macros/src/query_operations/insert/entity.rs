use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_insert_entity_function_tokens(
    _macro_data: &MacroTokens,
    table_schema_data: &str,
) -> TokenStream {
    let insert_entity_signature = quote! {
        async fn insert_entity<'canyon_lt, 'err_lt, Entity>(
            entity: &'canyon_lt mut Entity,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
        where
            Entity: canyon_sql::core::RowMapper
                + canyon_sql::query::bounds::EntityRuntimeInfo
                + Sync
                + 'canyon_lt
    };

    let insert_entity_with_signature = quote! {
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
    };

    let no_fields_to_insert_err =
        crate::query_operations::insert::__shared::no_fields_to_insert_err();

    let statement_initialization = quote! {
        use canyon_sql::query::querybuilder::QueryBuilderOps;

        let columns =
            <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>::field_columns();

        if columns.is_empty() {
            return #no_fields_to_insert_err;
        }

        let values = entity.field_values();

        let statement =
            canyon_sql::query::querybuilder::InsertQueryBuilder::new(
                #table_schema_data,
                db_conn.get_database_type()?,
            )
            .with_known_columns(columns);
    };

    let statement_execution = quote! {
        if let Some(primary_key_column) =
            <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>::primary_key_column()
        {
            let statement = statement
                .returning_columns(::core::iter::once(primary_key_column))
                .build()?;

            let primary_key = db_conn
                .query_one_for::<
                    <Entity as canyon_sql::query::bounds::EntityRuntimeInfo>::PrimaryKey
                >(
                    statement.sql(),
                    &values,
                )
                .await?;

            entity.set_primary_key(primary_key)?;
        } else {
            let statement = statement.build()?;

            db_conn
                .execute(statement.sql(), &values)
                .await?;
        }
    };

    quote! {
        #insert_entity_signature {
            let db_conn =
                canyon_sql::core::Canyon::instance()?.get_default_connection()?;

            #statement_initialization
            #statement_execution

            Ok(())
        }

        #insert_entity_with_signature {
            let db_conn = input;

            #statement_initialization
            #statement_execution

            Ok(())
        }
    }
}
