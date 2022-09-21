use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;
use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __update() CRUD operation
pub fn generate_update_tokens(macro_data: &MacroTokens) -> TokenStream {
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    // Retrives the fields of the Struct
    let fields = macro_data.get_struct_fields();

    // Retrieves the fields of the Struct as continuous String
    let column_names = macro_data.get_struct_fields_as_strings();

    let update_values = fields.iter().map( |ident| {
        quote! { &self.#ident }
    });
    let update_values_cloned = update_values.clone();

    let pk = macro_data.get_primary_key_annotation();

    if let Some(primary_key) = pk {
        quote! {
            /// Updates a database record that matches the current instance of a T type
            #vis async fn update(&self) -> () {
                let a = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__update(
                    #table_name,
                    #primary_key,
                    #column_names,
                    &[#(#update_values),*],
                    ""
                ).await
                .ok()
                .expect(
                    format!(
                        "Update operation failed for {:?}", 
                        &self
                    ).as_str()
                );
            }
    
            /// Updates a database record that matches the current instance of a T type
            #vis async fn update_datasource<'a>(&self, datasource_name: &'a str) {
                let a = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__update(
                    #table_name,
                    #primary_key,
                    #column_names,
                    &[#(#update_values_cloned),*],
                    datasource_name
                ).await
                .ok()
                .expect(
                    format!(
                        "Update operation failed for {:?}", 
                        &self
                    ).as_str()
                );
            }
        }
    } else {
        // If there's no primary key, update method over self won't be available.
        // Use instead the update associated function of the querybuilder
        quote! {}
    }

}

/// Generates the TokenStream for the __update() CRUD operation
/// returning a result containing the success or an error
pub fn generate_update_result_tokens(macro_data: &MacroTokens) -> TokenStream {
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    // Retrives the fields of the Struct
    let fields = macro_data.get_struct_fields();

    // Retrieves the fields of the Struct as continuous String
    let column_names = macro_data.get_struct_fields_as_strings();

    let update_values = fields.iter().map( |ident| {
        quote! { &self.#ident }
    });
    let update_values_cloned = update_values.clone();

    let pk = macro_data.get_primary_key_annotation();

    if let Some(primary_key) = pk {
        quote! {
            /// Updates a database record that matches
            /// the current instance of a T type, returning a result
            /// indicating a posible failure querying the database.
            #vis async fn update_result(&self) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>> {
                let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__update(
                    #table_name,
                    #primary_key,
                    #column_names,
                    &[#(#update_values),*],
                    ""
                ).await;
    
                if let Err(error) = result {
                    Err(error)
                } else { Ok(()) }
            }
    
            /// Updates a database record with the specified datasource that matches
            /// the current instance of a T type, returning a result with a posible 
            /// failure querying the database, or unit type in the success case.
            #vis async fn update_result_datasource<'a>(&self, datasource_name: &'a str) -> 
                Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>> 
            {
                let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__update(
                    #table_name,
                    #primary_key,
                    #column_names,
                    &[#(#update_values_cloned),*],
                    datasource_name
                ).await;
    
                if let Err(error) = result {
                    Err(error)
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
pub fn generate_update_query_tokens(macro_data: &MacroTokens) -> TokenStream {
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    quote! {
        /// TODO docs
        #vis fn update_query<'a>() -> query_elements::query_builder::QueryBuilder<'a, #ty> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__update_query(
                #table_name, ""
            )
        }

        /// TODO docs
        #vis fn update_query_datasource<'a>(datasource_name: &'a str) -> 
            query_elements::query_builder::QueryBuilder<'a, #ty> 
        {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__update_query(
                #table_name, datasource_name
            )
        }
    }
}