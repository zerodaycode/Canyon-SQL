use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;
use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __insert() CRUD operation
pub fn generate_insert_tokens(macro_data: &MacroTokens) -> TokenStream {

    // Destructure macro_tokens into raw data
    let (vis,ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    // Retrives the fields of the Struct
    let fields = macro_data.get_struct_fields();

    // Retrieves the fields of the Struct as continuous String
    let column_names = macro_data.get_struct_fields_as_strings();

    let insert_values = fields.iter().map( |ident| {
        quote! { &self.#ident }
    });


    quote! {
        #vis async fn insert(&self) -> () {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__insert(
                #table_name, 
                #column_names, 
                &[
                    #(#insert_values),*
                ]
            ).await;
        }
    }
}