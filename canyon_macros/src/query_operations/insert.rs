use proc_macro2::TokenStream;
use quote::quote;

use crate::query_operations::utils::helpers::*;
use crate::query_operations::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __insert() CRUD operation
pub fn generate_insert_tokens(macro_data: &MacroTokens) -> TokenStream {

    // Destructure macro_tokens into raw data
    let (vis,ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    // Retrives the fields of the Struct
    let fields = macro_data.get_struct_fields();

    // Retrieves the fields of the Struct as continuous String
    let column_names: String = fields.iter().map( |ident| {
        ident.to_string()
    }).collect::<Vec<String>>()
        .iter()
        .map( |column| column.to_owned() + ", " )
        .collect::<String>();
    
    let mut column_names_as_chars = column_names.chars();
    column_names_as_chars.next_back();
    column_names_as_chars.next_back();
    
    let column_names_pretty = column_names_as_chars.as_str();

    // Retrieves the actual data on the fields of the Struct
    let column_names: Vec<String> = fields.iter().map( |ident| {
        ident.to_string()
    }).collect::<Vec<String>>();

    println!("\nCOLUMN NAMES: {:?}", column_names);

    let insert_values: String = fields.iter().map( |ident| {
        ident.to_string()
    }).collect::<Vec<String>>()
        .iter()
        .map( |column| "&self.".to_owned() + &column.to_owned() + ", " )
        .collect::<String>();
    let mut insert_values_as_chars = insert_values.chars();
    insert_values_as_chars.next_back();
    insert_values_as_chars.next_back();
    
    let insert_values_pretty = column_names_as_chars.as_str();

    println!("\nINSERT VALUES PRETTY: {:?}", insert_values_pretty);

    quote! {
        // Insert into (as method)
        #vis async fn insert(&self) -> () {
            <#ty as CrudOperations<#ty>>::__insert(
                #table_name, 
                #column_names_pretty, 
                &[
                    // &self.#(#column_names),*
                ]
            )
                .await;
                // .as_response::<#ty>()[0].clone()
            () // TODO
        }
    }
}