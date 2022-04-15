use proc_macro2::{TokenStream, Ident, Span};
use quote::quote;

use crate::utils::helpers::*;

use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for build the __find_all() CRUD 
/// associated function
pub fn generate_find_all_tokens(macro_data: &MacroTokens) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis,ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis async fn find_all() -> Vec<#ty> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_all(
                #table_name
            )
            .await
            .as_response::<#ty>()
        }
    }
    
}

pub fn generate_find_all_query_tokens(macro_data: &MacroTokens) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis,ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis fn find_all_query() -> query_elements::query_builder::QueryBuilder<'static, #ty> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_all_query(
                #table_name
            )
        }
    }
   
}


/// Generates the TokenStream for build the __find_by_id() CRUD operation
pub fn generate_find_by_id_tokens(macro_data: &MacroTokens) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis,ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis async fn find_by_id(id: i32) -> #ty {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_by_id(#table_name, id)
                .await
                .as_response::<#ty>()[0].clone()
        }
    }
}

/// Generates the TokenStream for build the __search_by_foreign_key() CRUD 
/// associated function
pub fn generate_find_by_fk_tokens(macro_data: &MacroTokens, fk_data: String) -> TokenStream {

    let (vis,ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_struct(ty);

    let fk_table_column = &fk_data.split(",")
        .map( 
            |x| x.split(":").collect::<Vec<&str>>().get(1).unwrap().to_owned()
        ).collect::<Vec<&str>>()[1..].to_owned();

    let fk_table = make_related_table_plural(
        fk_table_column.get(0).unwrap().trim()
    );
    let fk_column = fk_table_column.get(1).unwrap().trim();
    // let fk_column_as_ident = Ident::new(
    //     fk_column, Span::call_site()
    // );

    // println!("FK_table: {:?}", &fk_table_column);
    println!("Table IN: {}", fk_table);
    println!("Column IN: {}", fk_column);

    quote! {
        #vis async fn search_by_fk<T>(value: &T) -> Vec<#ty> 
            where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
        {
            let lookage_value = value.get_fk_column(#fk_column).expect("Column not found");
            println!("\nLOOKAGE VALUE: {}\n", &lookage_value);
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::
            __search_by_foreign_key(#table_name, #fk_table, #fk_column, lookage_value)
                .await
                .as_response::<#ty>()
        }
    }
}

/// Helper to get the plural noun of a given table identifier
fn make_related_table_plural(singular: &str) -> String {
    // TODO
    singular.to_string()
}