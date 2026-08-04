use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_insert_entity_function_tokens(
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    let insert_entity_signature = __detail::generate_insert_entity_signature();
    let insert_entity_with_signature =
        __detail::generate_insert_entity_with_signature();

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

    Ok(quote! {
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
    })
}

mod __detail {
    pub(crate) fn generate_insert_entity_signature() -> proc_macro2::TokenStream {
        quote::quote! {
            fn insert_entity<'a, 'b, Entity>(
                entity: &'a mut Entity,
            ) -> impl ::core::future::Future<Output = Result<(), Box<dyn ::std::error::Error + Send + Sync + 'b>>>
            where
                Entity: canyon_sql::query::bounds::RowMapper + canyon_sql::query::bounds::EntityRuntimeInfo + Sync + 'a
        }
    }
    
    pub(crate) fn generate_insert_entity_with_signature() -> proc_macro2::TokenStream {
        quote::quote! {
            fn insert_entity_with<'a, 'b, Entity, I>(
                entity: &'a mut Entity,
                input: I,
            ) -> impl ::core::future::Future<Output = Result<(), Box<dyn ::std::error::Error + Send + Sync + 'b>>>
            where
                Entity: canyon_sql::query::bounds::RowMapper + canyon_sql::query::bounds::EntityRuntimeInfo + Sync + 'a,
                I: canyon_sql::connection::DbConnection + Send + 'a
        }
    }
}