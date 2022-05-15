use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;
use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __insert() CRUD operation
pub fn generate_insert_tokens(macro_data: &MacroTokens) -> TokenStream {

    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    // Retrieves the fields of the Struct as continuous String
    let column_names = macro_data.get_struct_fields_as_strings();

    // Retrives the fields of the Struct
    let fields = macro_data.get_struct_fields();

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

/// Generates the TokenStream for the __insert() CRUD operation, but being available
/// as a [`QueryBuilder`] object, and instead of being a method over some [`T`] type, 
/// as an associated function for [`T`]
/// 
/// This, also lets the user to have the option to be able to insert multiple
/// [`T`] objects in only one query
pub fn generate_insert_querybuider_tokens(macro_data: &MacroTokens) -> TokenStream {

    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    // Retrieves the fields of the Struct as continuous String
    let column_names = macro_data.get_struct_fields_as_strings();
    
    // Retrives the fields of the Struct
    let fields = macro_data.get_struct_fields();
    
    let macro_fields = fields.iter().map( |field| 
        quote! {
            &instance.#field 
        } 
    );


    quote! {
        #vis async fn insert_into(values: &[#ty]) -> () {
            use crate::tokio_postgres::types::ToSql;
            
            let mut final_values: Vec<Vec<Box<&(dyn ToSql + Sync)>>> = Vec::new();
            for instance in values.iter() {
                let intermediate: &[&(dyn ToSql + Sync)] = &[#(#macro_fields),*];
                
                let mut longer_lived: Vec<Box<&(dyn ToSql + Sync)>> = Vec::new();
                for value in intermediate.iter() {
                    longer_lived.push(Box::new(*value))
                }

                final_values.push(longer_lived)
            }
            
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__insert_querybuilder(
                #table_name, 
                #column_names, 
                &final_values
            ).await;
        }
    }
}