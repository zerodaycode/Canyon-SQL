use canyon_observer::CANYON_REGISTER_ENTITIES;
use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;

use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for build the __find_all() CRUD 
/// associated function
pub fn generate_find_all_tokens(macro_data: &MacroTokens) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis async fn find_all() -> Vec<#ty> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_all(
                #table_name
            ).await
                .ok()
                .unwrap()
                .to_entity::<#ty>()
        }
    }   
}

/// Generates the TokenStream for build the __find_all_result() CRUD 
/// associated function
pub fn generate_find_all_result_tokens(macro_data: &MacroTokens) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis async fn find_all_result() -> 
            Result<Vec<#ty>, canyon_sql::tokio_postgres::Error> 
        {
            let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_all(
                #table_name
            ).await;

            if let Err(error) = result {
                Err(error)
            } else {
                Ok(result.ok().unwrap().to_entity::<#ty>())
            }
        }
    }
    
}

/// Same as above, but with a [`query_elements::query_builder::QueryBuilder`]
pub fn generate_find_all_query_tokens(macro_data: &MacroTokens) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis fn find_all_query() -> query_elements::query_builder::QueryBuilder<'static, #ty> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_all_query(
                #table_name
            )
        }
    }
}

/// Performs a COUNT(*) query over some table
pub fn generate_count_tokens(macro_data: &MacroTokens<'_>) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis async fn count() -> i64 {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__count(
                #table_name
            ).await
                .ok()
                .unwrap()
        }
    }
}

/// Performs a COUNT(*) query over some table, returning a [`Result`] wrapping
/// a posible success or error coming from the database
pub fn generate_count_result_tokens(macro_data: &MacroTokens<'_>) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis async fn count_result() -> Result<i64, canyon_sql::tokio_postgres::Error> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__count(
                #table_name
            ).await
        }
    }
}

/// Generates the TokenStream for build the __find_by_id() CRUD operation
pub fn generate_find_by_id_tokens(macro_data: &MacroTokens) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis async fn find_by_id<N>(id: N) -> Option<#ty> 
            where N: canyon_sql::canyon_crud::bounds::IntegralNumber
        {
            let response = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_by_id(#table_name, id)
                .await;
                
            if response.as_ref().is_ok() {
                match response.as_ref().ok().unwrap() {
                    n if n.wrapper.len() == 0 => None,
                    _ => Some(
                        response
                            .ok()
                            .unwrap()
                            .to_entity::<#ty>()[0]
                            .clone()
                    )
                }
            } else { None }
        }
    }
}

/// Generates the TokenStream for build the __find_by_id() CRUD operation
pub fn generate_find_by_id_result_tokens(macro_data: &MacroTokens) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis async fn find_by_id_result<N>(id: N) -> 
            Result<Option<#ty>, canyon_sql::tokio_postgres::Error> 
                where N: canyon_sql::canyon_crud::bounds::IntegralNumber
        {
            let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_by_id(#table_name, id)
                .await;
                
            if let Err(error) = result {
                Err(error)
            } else { 
                match result.as_ref().ok().unwrap() {
                    n if n.wrapper.len() == 0 => Ok(None),
                    _ => Ok(Some(
                        result
                            .ok()
                            .unwrap()
                            .to_entity::<#ty>()[0]
                            .clone()
                    ))
                } 
            }
        }
    }
}

/// Generates the TokenStream for build the search by foreign key feature, also as a method instance
/// of a T type of as an associated function of same T type, but wrapped as a Result<T, Err>, representing
/// a posible failure querying the database, a bad or missing FK annotation or a missed ForeignKeyable
/// derive macro on the parent side of the relation
pub fn generate_find_by_foreign_key_tokens() -> Vec<TokenStream> {

    let mut foreign_keys_tokens = Vec::new();
    let mut column_name = String::new();

    for element in (*CANYON_REGISTER_ENTITIES).lock().unwrap().iter() {
        for field in &element.entity_fields {
            // Get the annotations attached to the entity, if some
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
                    let method_name = "search_".to_owned() + fk_table_column.get(0).unwrap().trim();

                    // Generate and identifier for the method based on the convention of "search_related_types" 
                    // where types is a placeholder for the plural name of the type referenced
                    let method_name_ident = proc_macro2::Ident::new(
                        &method_name, proc_macro2::Span::call_site()
                    );
                    let quoted_method_name: TokenStream = quote! { #method_name_ident }.into();

                    // Converts a database table name generated by convection (lower case separated
                    // by underscores) to the Rust struct identifier convenction
                    let fk_ty = database_table_name_to_struct_ident(fk_table);

                    // The ident for the field that holds the FK relation
                    let field_ident = proc_macro2::Ident::new(
                        &field.field_name, proc_macro2::Span::call_site()
                    );
                    let field_value = quote! { &self.#field_ident };

                    foreign_keys_tokens.push(
                        quote! {
                            /// Searches the parent entity (if exists) for this type
                            pub async fn #quoted_method_name(&self) -> Option<#fk_ty> {
                                let lookage_value = #field_value.to_string();
                                let response = <#fk_ty as canyon_sql::canyon_crud::crud::CrudOperations<#fk_ty>>::
                                    __search_by_foreign_key(#fk_table, #fk_column, &lookage_value)
                                        .await;
                                
                                if response.as_ref().is_ok() {
                                    match response.as_ref().ok().unwrap() {
                                        n if n.wrapper.len() == 0 => None,
                                        _ => Some(
                                            response
                                                .ok()
                                                .unwrap()
                                                .to_entity::<#fk_ty>()[0]
                                                .clone()
                                        )
                                    }
                                } else { None }
                            }
                            
                            /// Searches the parent entity (if exists) for the type &T passed in
                            /// 
                            /// Note that if you pass an instance of some ForeignKeyable type that
                            /// does not matches the other side of the relation, an error will be
                            /// generated
                            pub async fn belongs_to<T>(value: &T) -> Option<#fk_ty> 
                                where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
                            {
                                let lookage_value = value.get_fk_column(#fk_column).expect("Column not found");
                                let response = <#fk_ty as canyon_sql::canyon_crud::crud::CrudOperations<#fk_ty>>::
                                    __search_by_foreign_key(#fk_table, #fk_column, &lookage_value)
                                        .await;
                                
                                if response.as_ref().is_ok() {
                                    match response.as_ref().ok().unwrap() {
                                        n if n.wrapper.len() == 0 => None,
                                        _ => Some(
                                            response
                                                .ok()
                                                .unwrap()
                                                .to_entity::<#fk_ty>()[0]
                                                .clone()
                                        )
                                    }
                                } else { None }
                            }
                        }
                    );
                }
            }
        }
    }

    foreign_keys_tokens
}

/// Generates the TokenStream for build the search by foreign key feature, also as a method instance
/// of a T type of as an associated function of same T type
pub fn generate_find_by_foreign_key_result_tokens() -> Vec<TokenStream> {

    let mut foreign_keys_tokens = Vec::new();
    let mut column_name = String::new();

    for element in (*CANYON_REGISTER_ENTITIES).lock().unwrap().iter() {
        for field in &element.entity_fields {
            // Get the annotations attached to the entity, if some
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
                        fk_table_column.get(0).unwrap().trim() + 
                        "_result";

                    // Generate and identifier for the method based on the convention of "search_related_types" 
                    // where types is a placeholder for the plural name of the type referenced
                    let method_name_ident = proc_macro2::Ident::new(
                        &method_name, proc_macro2::Span::call_site()
                    );
                    let quoted_method_name: TokenStream = quote! { #method_name_ident }.into();

                    // Converts a database table name generated by convection (lower case separated
                    // by underscores) to the Rust struct identifier convenction
                    let fk_ty = database_table_name_to_struct_ident(fk_table);

                    // The ident for the field that holds the FK relation
                    let field_ident = proc_macro2::Ident::new(
                        &field.field_name, proc_macro2::Span::call_site()
                    );
                    let field_value = quote! { &self.#field_ident };

                    foreign_keys_tokens.push(
                        quote! {
                            /// Searches the parent entity (if exists) for this type
                            pub async fn #quoted_method_name(&self) -> 
                                Result<Option<#fk_ty>, canyon_sql::tokio_postgres::Error> 
                            {
                                let lookage_value = #field_value.to_string();
                                let result = <#fk_ty as canyon_sql::canyon_crud::crud::CrudOperations<#fk_ty>>::
                                    __search_by_foreign_key(#fk_table, #fk_column, &lookage_value)
                                        .await;
                                
                                if let Err(error) = result {
                                    Err(error)
                                } else { 
                                    match result.as_ref().ok().unwrap() {
                                        n if n.wrapper.len() == 0 => Ok(None),
                                        _ => Ok(Some(
                                            result
                                                .ok()
                                                .unwrap()
                                                .to_entity::<#fk_ty>()[0]
                                                .clone()
                                        ))
                                    } 
                                }
                            }
                            
                            /// Searches the parent entity (if exists) for the type &T passed in
                            /// 
                            /// Note that if you pass an instance of some ForeignKeyable type that
                            /// does not matches the other side of the relation, an error will be
                            /// generated
                            pub async fn belongs_to_result<T>(value: &T) ->
                                Result<Option<#fk_ty>, canyon_sql::tokio_postgres::Error> 
                                    where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
                            {
                                let lookage_value = value.get_fk_column(#fk_column).expect("Column not found");
                                let result = <#fk_ty as canyon_sql::canyon_crud::crud::CrudOperations<#fk_ty>>::
                                    __search_by_foreign_key(#fk_table, #fk_column, &lookage_value)
                                        .await;
                        
                                if let Err(error) = result {
                                    Err(error)
                                } else { 
                                    match result.as_ref().ok().unwrap() {
                                        n if n.wrapper.len() == 0 => Ok(None),
                                        _ => Ok(Some(
                                            result
                                                .ok()
                                                .unwrap()
                                                .to_entity::<#fk_ty>()[0]
                                                .clone()
                                        ))
                                    } 
                                }
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
    for element in (*CANYON_REGISTER_ENTITIES).lock().unwrap().iter() {
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
                            #vis async fn #quoted_method_name<T>(value: &T) -> Result<Vec<#ty>, canyon_sql::tokio_postgres::Error> 
                                where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
                            {
                                let lookage_value = value.get_fk_column(#lookage_value_column).expect("Column not found");
                                let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::
                                    __search_by_reverse_side_foreign_key(#table_name, #column_name, lookage_value)
                                        .await;

                                if let Err(error) = result {
                                    Err(error)
                                } else { 
                                    Ok(
                                        result
                                            .ok()
                                            .unwrap()
                                            .to_entity::<#ty>()
                                    )
                                }
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
/// associated function, but wrapped as a Result<T, Err>, representing
/// a posible failure querying the database, a bad or missing FK annotation or a missed ForeignKeyable
/// derive macro on the parent side of the relation
pub fn generate_find_by_reverse_foreign_key_result_tokens(macro_data: &MacroTokens) -> Vec<TokenStream> {

    let mut foreign_keys_tokens = Vec::new();

    let (vis, ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);
    let mut column_name = String::new();
    let mut lookage_value_column = String::new();

    // Find what relation belongs to the data passed in
    for element in (*CANYON_REGISTER_ENTITIES).lock().unwrap().iter() {
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
                    let method_name = "search_by__".to_owned() + 
                        fk_table_column.get(0).unwrap().trim()
                        + "_result";

                    // Generate and identifier for the method based on the convention of "search_by__" (note the double underscore)
                    // plus the 'table_name' property of the ForeignKey annotation
                    let method_name_ident = proc_macro2::Ident::new(
                        &method_name, proc_macro2::Span::call_site()
                    );
                    let quoted_method_name: TokenStream = quote! { #method_name_ident }.into();

                    foreign_keys_tokens.push(
                        quote! {
                            #vis async fn #quoted_method_name<T>(value: &T) -> 
                                Result<canyon_sql::result::DatabaseResult<#ty>, canyon_sql::tokio_postgres::Error> 
                                    where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
                            {
                                let lookage_value = value.get_fk_column(#lookage_value_column).expect("Column not found");
                                <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::
                                    __search_by_reverse_side_foreign_key(#table_name, #column_name, lookage_value)
                                        .await
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
fn _make_related_table_plural(singular: &str) -> String {
    // TODO Generate the correct forms of the plural for a given identifier
    // For brevity, and for now, just adds an 's' to the end of the noun
    singular.to_owned() + "s"
}