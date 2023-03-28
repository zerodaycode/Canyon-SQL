use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __update() CRUD operation
pub fn generate_update_tokens(macro_data: &MacroTokens, table_schema_data: &String) -> TokenStream {
    let ty = macro_data.ty;

    let update_columns = macro_data.get_column_names_pk_parsed();

    // Retrieves the fields of the Struct
    let fields = macro_data.get_struct_fields();

    let mut vec_columns_values: Vec<String> = Vec::new();
    for (i, column_name) in update_columns.iter().enumerate() {
        let column_equal_value = format!("{} = ${}", column_name.to_owned(), i + 2);
        vec_columns_values.push(column_equal_value)
    }

    let str_columns_values = vec_columns_values.join(", ");

    let update_values = fields.iter().map(|ident| {
        quote! { &self.#ident }
    });
    let update_values_cloned = update_values.clone();

    if let Some(primary_key) = macro_data.get_primary_key_annotation() {
        let pk_index = macro_data
            .get_pk_index()
            .expect("Update method failed to retrieve the index of the primary key");

        quote! {
            /// Updates a database record that matches
            /// the current instance of a T type, returning a result
            /// indicating a possible failure querying the database.
            async fn update(&self) -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>> {
                let stmt = format!(
                    "UPDATE {} SET {} WHERE {} = ${:?}",
                    #table_schema_data, #str_columns_values, #primary_key, #pk_index + 1
                );
                let update_values: &[&dyn canyon_sql::crud::bounds::QueryParameter<'_>] = &[#(#update_values),*];

                <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                    stmt, update_values, ""
                ).await?;
                
                Ok(())
            }


            /// Updates a database record that matches
            /// the current instance of a T type, returning a result
            /// indicating a possible failure querying the database with the
            /// specified datasource
            async fn update_datasource<'a>(&self, datasource_name: &'a str)
                -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>
            {
                let stmt = format!(
                    "UPDATE {} SET {} WHERE {} = ${:?}",
                    #table_schema_data, #str_columns_values, #primary_key, #pk_index + 1
                );
                let update_values: &[&dyn canyon_sql::crud::bounds::QueryParameter<'_>] = &[#(#update_values_cloned),*];

                <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                    stmt, update_values, datasource_name
                ).await?;

                Ok(())
            }
        }
    } else {
        // If there's no primary key, update method over self won't be available.
        // Use instead the update associated function of the querybuilder

        // TODO Returning an error should be a provisional way of doing this
        quote! {
            async fn update(&self)
                -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>
            {
                Err(
                    std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "You can't use the 'update' method on a \
                        CanyonEntity that does not have a #[primary_key] annotation. \
                        If you need to perform an specific search, use the Querybuilder instead."
                    ).into_inner().unwrap()
                )
            }

            async fn update_datasource<'a>(&self, datasource_name: &'a str)
                -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>
            {
                Err(
                    std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "You can't use the 'update_datasource' method on a \
                        CanyonEntity that does not have a #[primary_key] annotation. \
                        If you need to perform an specific search, use the Querybuilder instead."
                    ).into_inner().unwrap()
                )
            }
        }
    }
}

/// Generates the TokenStream for the __update() CRUD operation
/// being the query generated with the [`QueryBuilder`]
pub fn generate_update_query_tokens(
    macro_data: &MacroTokens,
    table_schema_data: &String,
) -> TokenStream {
    let ty = macro_data.ty;

    quote! {
        /// Generates a [`canyon_sql::query::UpdateQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `UPDATE table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        fn update_query<'a>() -> canyon_sql::query::UpdateQueryBuilder<'a, #ty> {
            canyon_sql::query::UpdateQueryBuilder::new(#table_schema_data, "")
        }

        /// Generates a [`canyon_sql::query::UpdateQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `UPDATE table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        ///
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the [`&str`]
        /// passed as parameter.
        fn update_query_datasource<'a>(datasource_name: &'a str) -> canyon_sql::query::UpdateQueryBuilder<'a, #ty> {
            canyon_sql::query::UpdateQueryBuilder::new(#table_schema_data, datasource_name)
        }
    }
}
