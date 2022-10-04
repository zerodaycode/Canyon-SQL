use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;
use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __update() CRUD operation
pub fn generate_update_tokens(macro_data: &MacroTokens, table_schema_data: &String) -> TokenStream {
    let ty = macro_data.ty;

    let update_columns = macro_data.get_column_names_pk_parsed();

    // Retrives the fields of the Struct
    let fields = macro_data.get_struct_fields();

    let mut vec_columns_values:Vec<String> = Vec::new();
    for (i, column_name) in update_columns.iter().enumerate() {
        let column_equal_value = format!(
            "{} = ${}", column_name.to_owned(), i + 2
        );
        vec_columns_values.push(column_equal_value)
    }

    let str_columns_values = vec_columns_values.join(", ");

    let update_values = fields.iter().map( |ident| {
        quote! { &self.#ident }
    });
    let update_values_cloned = update_values.clone();
    let update_values_cloned2 = update_values.clone();
    let update_values_cloned3 = update_values.clone();

    if let Some(primary_key) = macro_data.get_primary_key_annotation() {
        let pk_index = macro_data.get_pk_index()
            .expect("Update method failed to retrieve the index of the primary key");
        
        quote! {
            /// Updates a database record that matches the current instance of a T type
            async fn update(&self) {
                let stmt = format!(
                    "UPDATE {} SET {} WHERE {} = ${:?}",
                    #table_schema_data, #str_columns_values, #primary_key, #pk_index + 1
                );
                let update_values: &[&dyn canyon_sql::bounds::QueryParameters<'_>] = &[#(#update_values),*];

                let a = <#ty as canyon_sql::canyon_crud::crud::Transaction<#ty>>::query(
                    stmt, update_values, ""
                ).await
                .ok()
                .expect(
                    format!(
                        "Update operation failed for {:?}", 
                        &self
                    ).as_str()
                );
            }

            /// Updates a database record that matches
            /// the current instance of a T type, returning a result
            /// indicating a posible failure querying the database.
            async fn update_result(&self) -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>> {
                let stmt = format!(
                    "UPDATE {} SET {} WHERE {} = ${:?}",
                    #table_schema_data, #str_columns_values, #primary_key, #pk_index + 1
                );
                let update_values: &[&dyn canyon_sql::bounds::QueryParameters<'_>] = &[#(#update_values_cloned),*];

                let result = <#ty as canyon_sql::canyon_crud::crud::Transaction<#ty>>::query(
                    stmt, update_values, ""
                ).await;

                if let Err(e) = result {
                    Err(e)
                } else { Ok(()) }
            }
    
            /// Updates a database record that matches the current instance of a T type
            async fn update_datasource<'a>(&self, datasource_name: &'a str) {
                let stmt = format!(
                    "UPDATE {} SET {} WHERE {} = ${:?}",
                    #table_schema_data, #str_columns_values, #primary_key, #pk_index + 1
                );
                let update_values: &[&dyn canyon_sql::bounds::QueryParameters<'_>] = &[#(#update_values_cloned2),*];

                let a = <#ty as canyon_sql::canyon_crud::crud::Transaction<#ty>>::query(
                    stmt, update_values, datasource_name
                ).await
                .ok()
                .expect(
                    format!(
                        "Update operation failed for {:?}", 
                        &self
                    ).as_str()
                );
            }

            /// Updates a database record that matches
            /// the current instance of a T type, returning a result
            /// indicating a posible failure querying the database with the
            /// specified datasource
            async fn update_result_datasource<'a>(&self, datasource_name: &'a str) 
                -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>
            {
                let stmt = format!(
                    "UPDATE {} SET {} WHERE {} = ${:?}",
                    #table_schema_data, #str_columns_values, #primary_key, #pk_index + 1
                );
                let update_values: &[&dyn canyon_sql::bounds::QueryParameters<'_>] = &[#(#update_values_cloned3),*];

                let result = <#ty as canyon_sql::canyon_crud::crud::Transaction<#ty>>::query(
                    stmt, update_values, datasource_name
                ).await;

                if let Err(e) = result {
                    Err(e)
                } else { Ok(()) }
            }
        }
    } else {
        // If there's no primary key, update method over self won't be available.
        // Use instead the update associated function of the querybuilder
        quote! {}
    }

}

/// Generates the TokenStream for the __update() CRUD operation
/// being the query generated with the [`QueryBuilder`]
pub fn generate_update_query_tokens(macro_data: &MacroTokens, table_schema_data: &String) -> TokenStream {
    let ty = macro_data.ty;

    quote! {
        /// TODO docs
        fn update_query<'a>() -> query_elements::query_builder::QueryBuilder<'a, #ty> {
            query_elements::query::Query::new(format!("UPDATE {}", #table_schema_data), "")
        }

        /// TODO docs
        fn update_query_datasource<'a>(datasource_name: &'a str) 
            -> query_elements::query_builder::QueryBuilder<'a, #ty> 
        {
            query_elements::query::Query::new(format!("UPDATE {}", #table_schema_data), datasource_name)
        }
    }
}