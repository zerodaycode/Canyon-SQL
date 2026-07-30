use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn generate_delete_entity_tokens(table_schema_data: &str) -> TokenStream {
    let delete_entity_signature = quote! {
        async fn delete_entity<'canyon, 'err, Entity>(entity: &'canyon Entity)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err>>
        where Entity: canyon_sql::core::RowMapper
            + canyon_sql::query::bounds::EntityRuntimeInfo<'canyon>
            + Sync
            + 'canyon
    };

    let delete_entity_with_signature = quote! {
        async fn delete_entity_with<'canyon, 'err, Entity, Input>(entity: &'canyon Entity, input: Input)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err>>
        where
            Entity: canyon_sql::core::RowMapper
                + canyon_sql::query::bounds::EntityRuntimeInfo<'canyon>
                + Sync
                + 'canyon,
            Input: canyon_sql::connection::DbConnection + Send + 'canyon
    };

    let delete_entity_body = __detail::generate_delete_entity_body(table_schema_data);
    let delete_entity_with_body = __detail::generate_delete_entity_with_body(&table_schema_data);

    quote! {
        #delete_entity_signature { #delete_entity_body }
        #delete_entity_with_signature { #delete_entity_with_body }
    }
}

mod __detail {
    use crate::query_operations::consts;
    use proc_macro2::TokenStream;
    use quote::quote;

    pub(crate) fn generate_delete_entity_body(table_schema_data: &str) -> TokenStream {
        let delete_stmt = generate_delete_query(table_schema_data);
        let no_pk_err = consts::generate_no_pk_error();
        let default_db_conn_and_type_tokens =
            consts::generate_default_db_conn_and_type_tokens();

        quote! {
            if let Some(primary_key) = entity.primary_key() {
                #default_db_conn_and_type_tokens
                #delete_stmt
                let _ = default_db_conn.execute(&delete_stmt.as_ref(), &[pk_actual_value]).await?;
                Ok(())
            } else {
                #no_pk_err
            }
        }
    }

    pub(crate) fn generate_delete_entity_with_body(table_schema_data: &str) -> TokenStream {
        let delete_stmt = generate_delete_query(table_schema_data);
        let no_pk_err = consts::generate_no_pk_error();

        quote! {
            if let Some(primary_key) = entity.primary_key() {
                let db_type = input.get_database_type()?;
                #delete_stmt
                let _ = input.execute(&delete_stmt.as_ref(), &[pk_actual_value]).await?;
                Ok(())
            } else {
                #no_pk_err
            }
        }
    }

    fn generate_delete_query(table_schema_data: &str) -> TokenStream {
        quote! {
            use canyon_sql::query::querybuilder::{QueryBuilderOps, DeleteQueryBuilderOps};

            let pk_actual_value = entity.primary_key_actual_value();
            let delete_stmt =
                canyon_sql::query::querybuilder::DeleteQueryBuilder::new_for(
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
}
