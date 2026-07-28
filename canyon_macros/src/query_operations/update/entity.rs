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

    pub(crate) fn generate_update_entity_body(table_schema_data: &str) -> TokenStream {
        let update_entity_core_logic = generate_update_entity_pk_body_logic(table_schema_data);
        let no_pk_err = __err::generate_no_pk_err();

        quote! {
            if let Some(primary_key) = entity.primary_key() {
                #update_entity_core_logic

                let default_db_conn = canyon_sql::core::Canyon::instance()?
                    .get_default_connection()?;
                let _ = default_db_conn.execute(&stmt, &update_values).await?;
                Ok(())
            } else {
                #no_pk_err
            }
        }
    }

    pub(crate) fn generate_update_entity_with_body(table_schema_data: &str) -> TokenStream {
        let update_entity_core_logic = generate_update_entity_pk_body_logic(table_schema_data);
        let no_pk_err = __err::generate_no_pk_err();

        quote! {
            if let Some(primary_key) = entity.primary_key() {
                #update_entity_core_logic

                let _ = input.execute(&stmt, &update_values).await?;
                Ok(())
            } else {
                #no_pk_err
            }
        }
    }

    fn generate_update_entity_pk_body_logic(table_schema_data: &str) -> TokenStream {
        quote! {
            let pk_actual_value = entity.primary_key_actual_value();
            let update_columns = entity.fields_names();
            let update_values_pk_parsed = entity.fields_actual_values();

            let mut vec_columns_values: Vec<String> = Vec::new();
            for (i, column_name) in update_columns.to_vec().iter().enumerate() {
                let column_equal_value = format!("{} = ${}", column_name, i + 2);
                vec_columns_values.push(column_equal_value)
            }
            let col_vals_placeholders = vec_columns_values.join(", ");

            // Efficiently build argument list: pk first, then values
            let mut update_values: Vec<&dyn canyon_sql::query::QueryParameter> =
                Vec::with_capacity(1 + update_values_pk_parsed.len());
            update_values.push(pk_actual_value);
            update_values.extend(update_values_pk_parsed);

            let stmt = format!(
                "UPDATE {} SET {} WHERE {:?} = $1",
                #table_schema_data, col_vals_placeholders, primary_key
            );
        }
    }
}
