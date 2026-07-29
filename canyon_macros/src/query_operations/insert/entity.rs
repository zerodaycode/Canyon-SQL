use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_insert_entity_function_tokens(
    _macro_data: &MacroTokens,
    table_schema_data: &str,
) -> TokenStream {
    let insert_entity_signature = quote! {
        async fn insert_entity<'canyon_lt, 'err_lt, Entity>(entity: &'canyon_lt mut Entity)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
        where Entity: canyon_sql::core::RowMapper
            + canyon_sql::query::bounds::EntityRuntimeInfo<'canyon_lt>
            + Sync
            + 'canyon_lt
    };

    let insert_entity_with_signature = quote! {
        async fn insert_entity_with<'canyon_lt, 'err_lt, Entity, Input>(entity: &'canyon_lt mut Entity, input: Input)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
        where
            Entity: canyon_sql::core::RowMapper
                + canyon_sql::query::bounds::EntityRuntimeInfo<'canyon_lt>
                + Sync
                + 'canyon_lt,
            Input: canyon_sql::connection::DbConnection + Send + 'canyon_lt
    };

    let no_fields_to_insert_err =
        crate::query_operations::insert::__shared::no_fields_to_insert_err();

    let stmt_ctr = quote! {
        use canyon_sql::query::querybuilder::{InsertQueryBuilderOps, QueryBuilderOps};

        let insert_entity_columns = entity.fields_as_column_refs();

        if insert_entity_columns.is_empty() {
            return #no_fields_to_insert_err;
        }

        let values = entity.fields_actual_values();

        let stmt = canyon_sql::query::querybuilder::InsertQueryBuilder::new(
            #table_schema_data,
            db_conn.get_database_type()?,
        )
        .with_known_columns(insert_entity_columns)
    };

    quote! {
        #insert_entity_signature {
            let db_conn = canyon_sql::core::Canyon::instance()?.get_default_connection()?;
            #stmt_ctr;

            if let Some(pk) = entity.primary_key_as_column_ref() {
                let stmt = stmt
                    .returning_columns(::core::iter::once(pk))
                    .build()?;

                let pk = db_conn
                    .query_one_for::<<Entity as canyon_sql::query::bounds::EntityRuntimeInfo>::PrimaryKeyType>(stmt.sql(), &values)
                    .await?;

                entity.set_primary_key_actual_value(pk)?;
            } else {
                let stmt = stmt.build()?;
                let _ = db_conn.execute(stmt.sql(), &values).await?;
            }

            Ok(())
        }

        #insert_entity_with_signature {
            let db_conn = input;
            #stmt_ctr;

            if let Some(pk) = entity.primary_key_as_column_ref() {
                let stmt = stmt
                    .returning_columns(::core::iter::once(pk))
                    .build()?;

                let pk = db_conn
                    .query_one_for::<<Entity as canyon_sql::query::bounds::EntityRuntimeInfo>::PrimaryKeyType>(stmt.sql(), &values)
                    .await?;

                entity.set_primary_key_actual_value(pk)?;
            } else {
                let stmt = stmt.build()?;
                let _ = db_conn.execute(stmt.sql(), &values).await?;
            }

            Ok(())
        }
    }
}
