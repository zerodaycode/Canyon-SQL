use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;
use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __update() CRUD operation
pub fn generate_update_tokens(macro_data: &MacroTokens) -> TokenStream {

    // Destructure macro_tokens into raw data
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


    quote! {
        /// Updates a database record that matches
        /// the current instance of a T type
        #vis async fn update(&self) -> () {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__update(
                #table_name,
                #column_names,
                &[
                    #(#update_values),*
                ]
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
}

/// Generates the TokenStream for the __update() CRUD operation
/// returning a result containing the success or an error
pub fn generate_update_result_tokens(macro_data: &MacroTokens) -> TokenStream {

    // Destructure macro_tokens into raw data
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


    quote! {
        /// Updates a database record that matches
        /// the current instance of a T type, returning a result
        /// indicating a posible failure querying the database.
        #vis async fn update_result(&self) ->
            Result<canyon_sql::result::DatabaseResult<#ty>, canyon_sql::tokio_postgres::Error>
        {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__update(
                #table_name,
                #column_names,
                &[
                    #(#update_values),*
                ]
            ).await
        }
    }
}

/// Generates the TokenStream for the __update() CRUD operation
/// being the query generated with the [`QueryBuilder`]
pub fn generate_update_query_tokens(macro_data: &MacroTokens) -> TokenStream {

    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis fn update_query() -> query_elements::query_builder::QueryBuilder<'static, #ty> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__update_query(
                #table_name
            )
        }
    }
}