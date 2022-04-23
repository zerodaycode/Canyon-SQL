use canyon_observer::CANYON_REGISTER_ENTITIES;
use proc_macro2::TokenStream;
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


/// Generates the TokenStream for build the __search_by_foreign_key() 
/// CRUD associated function
pub fn generate_find_by_foreign_key_tokens() -> Vec<TokenStream> {

    let mut foreign_keys_tokens = Vec::new();
    let mut column_name = String::new();

    // Find what relation belongs to the data passed in
    for element in unsafe { &CANYON_REGISTER_ENTITIES } {
        for field in &element.entity_fields {
            // Get the annotations
            if field.annotation.is_some() {
                let annotation = field.annotation.as_ref().unwrap();
                if annotation.starts_with("Annotation: ForeignKey") {
                    column_name.push_str(&field.field_name);
                    let fk_table_column = &annotation.split(",")
                        .map( |x| 
                            x.split(":")
                            .collect::<Vec<&str>>()
                            .get(1)
                            .unwrap()
                            .to_owned()
                        ).collect::<Vec<&str>>()[1..].to_owned();
            
                    let fk_table = fk_table_column.get(0).unwrap().trim();
                    let fk_column = fk_table_column.get(1).unwrap().trim();
                    let method_name = "search_".to_owned() + 
                        &make_related_table_plural(fk_table_column.get(0).unwrap().trim());

                    // Generate and identifier for the method based on the convention of "search_related_types" 
                    // where types is a placeholder for the plural name of the type referenced
                    let method_name_ident = proc_macro2::Ident::new(
                        &method_name, proc_macro2::Span::call_site()
                    );
                    let quoted_method_name: TokenStream = quote! { #method_name_ident }.into();

                    // For now, we rollback the way when Canyon convert's the struct name to a decided by
                    // convenction database table name
                    let fk_ty = database_table_name_to_struct_ident(fk_table);

                    foreign_keys_tokens.push(
                        quote! {
                            pub async fn #quoted_method_name<T>(value: &T) -> Vec<#fk_ty> 
                                where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
                            {
                                let lookage_value = value.get_fk_column(#fk_column).expect("Column not found");
                                <#fk_ty as canyon_sql::canyon_crud::crud::CrudOperations<#fk_ty>>::
                                    __search_by_foreign_key(#fk_table, #fk_column, lookage_value)
                                        .await
                                        .as_response::<#fk_ty>()
                            }
                        }
                    );
                }
            }
        }
    }

    foreign_keys_tokens
}

/// Generates the TokenStream for build the __search_by_foreign_key() CRUD 
/// associated function
pub fn generate_find_by_reverse_foreign_key_tokens(macro_data: &MacroTokens) -> Vec<TokenStream> {

    let mut foreign_keys_tokens = Vec::new();

    let (vis, ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);
    let mut column_name = String::new();
    let mut lookage_value_column = String::new();

    // Find what relation belongs to the data passed in
    for element in unsafe { &CANYON_REGISTER_ENTITIES } {
        for field in &element.entity_fields {
            // Get the annotations
            if field.annotation.is_some() {
                let annotation = field.annotation.as_ref().unwrap();
                if annotation.starts_with("Annotation: ForeignKey") {
                    column_name.push_str(&field.field_name);
                    let fk_table_column = &annotation.split(",")
                        .map( |x| 
                            x.split(":")
                            .collect::<Vec<&str>>()
                            .get(1)
                            .unwrap()
                            .to_owned()
                        ).collect::<Vec<&str>>()[1..].to_owned();
            
                    lookage_value_column.push_str(fk_table_column.get(1).unwrap().trim());
                    let method_name = "search_by__".to_owned() + fk_table_column.get(0).unwrap().trim();

                    // Generate and identifier for the method based on the convention of "search_by__" (note the double underscore)
                    // plus the 'table_name' property of the ForeignKey annotation
                    let method_name_ident = proc_macro2::Ident::new(
                        &method_name, proc_macro2::Span::call_site()
                    );
                    let quoted_method_name: TokenStream = quote! { #method_name_ident }.into();

                    foreign_keys_tokens.push(
                        quote! {
                            #vis async fn #quoted_method_name<T>(value: &T) -> Vec<#ty> 
                                where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
                            {
                                let lookage_value = value.get_fk_column(#lookage_value_column).expect("Column not found");
                                <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::
                                    __search_by_reverse_side_foreign_key(#table_name, #column_name, lookage_value)
                                        .await
                                        .as_response::<#ty>()
                            }
                        }
                    );
                }
            }
        }
    }

    foreign_keys_tokens
}



/// Helper to get the plural noun of a given table identifier
fn make_related_table_plural(singular: &str) -> String {
    // TODO Generate the correct forms of the plural for a given identifier
    // For brevity, and for now, just adds an 's' to the end of the noun
    singular.to_owned() + "s"
}