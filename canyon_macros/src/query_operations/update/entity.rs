use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn generate_update_entity_tokens(table_schema_data: &str) -> TokenStream {
    let update_entity_signature = quote! {
        async fn update_entity<'canyon_lt, 'err_lt, Entity>(entity: &'canyon_lt Entity)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
        where Entity: canyon_sql::core::RowMapper
            + canyon_sql::query::bounds::Inspectionable<'canyon_lt>
            + Sync
            + 'canyon_lt
    };

    let update_entity_with_signature = quote! {
        async fn update_entity_with<'canyon_lt, 'err_lt, Entity, Input>(entity: &'canyon_lt Entity, input: Input)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
        where
            Entity: canyon_sql::core::RowMapper
                + canyon_sql::query::bounds::Inspectionable<'canyon_lt>
                + Sync
                + 'canyon_lt,
            Input: canyon_sql::connection::DbConnection + Send + 'canyon_lt
    };

    let update_entity_body = __details::generate_update_entity_body(table_schema_data);
    let update_entity_with_body = __details::generate_update_entity_with_body(table_schema_data);

    quote! {
        #update_entity_signature { #update_entity_body }
        #update_entity_with_signature { #update_entity_with_body }
    }
}

mod __details {
    use crate::query_operations::update::__err;
    use proc_macro2::TokenStream;
    use quote::quote;
    use crate::query_operations::consts;

    pub(crate) fn generate_update_entity_body(table_schema_data: &str) -> TokenStream {
        let update_query = generate_update_query(table_schema_data);
        let default_db_conn_and_type_tokens = consts::generate_default_db_conn_and_type_tokens();
        let no_pk_err = __err::generate_no_pk_err();

        quote! {
            if let Some(primary_key) = entity.primary_key() {
                #default_db_conn_and_type_tokens

                let pk_actual_value = entity.primary_key_actual_value();
                let mut update_values = entity.fields_actual_values();
                update_values.push(pk_actual_value);

                #update_query

                let _ = default_db_conn.execute(query.as_ref(), &update_values).await?;
                Ok(())
            } else {
                #no_pk_err
            }
        }
    }

    pub(crate) fn generate_update_entity_with_body(table_schema_data: &str) -> TokenStream {
        let update_query = generate_update_query(table_schema_data);
        let no_pk_err = __err::generate_no_pk_err();

        quote! {
            if let Some(primary_key) = entity.primary_key() {
                let pk_actual_value = entity.primary_key_actual_value();
                let mut update_values = entity.fields_actual_values();
                update_values.push(pk_actual_value);

                let db_type = input.get_database_type()?;
                #update_query

                let _ = input.execute(query.as_ref(), &update_values).await?;
                Ok(())
            } else {
                #no_pk_err
            }
        }
    }

    fn generate_update_query(table_schema_data: &str, ) -> TokenStream {
        quote! {
            let update_columns = entity.fields_as_column_refs();

            use canyon_sql::query::querybuilder::{QueryBuilderOps, UpdateQueryBuilderOps};
            let query = canyon_sql::query::querybuilder::UpdateQueryBuilder::new_for(
                #table_schema_data, // TODO: construct a const value
                db_type,
            )
                .set(update_columns)?
                .r#where(
                    primary_key,
                    canyon_sql::query::operators::Operator::Eq,
                )
                .build()?;
        }
    }
}
