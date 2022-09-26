use canyon_observer::CANYON_REGISTER_ENTITIES;

use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;
use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for build the __find_all() CRUD 
/// associated function
pub fn generate_find_all_tokens(macro_data: &MacroTokens<'_>) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_struct(ty);

    quote! {
        /// Performns a `SELECT * FROM table_name`, where `table_name` it's
        /// the name of your entity but converted to the corresponding
        /// database convention. P.ej. PostgreSQL preferes table names declared
        /// with snake_case identifiers.
        #vis async fn find_all<'a>() -> Vec<#ty>{
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::
            __find_all(
                #table_name,
                ""
            ).await
                .ok()
                .unwrap()
                .get_entities::<#ty>()
        }

        /// Performns a `SELECT * FROM table_name`, where `table_name` it's
        /// the name of your entity but converted to the corresponding
        /// database convention. P.ej. PostgreSQL preferes table names declared
        /// with snake_case identifiers.
        /// 
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the [`&str`] 
        /// passed as parameter.
        #vis async fn find_all_datasource<'a>(datasource_name: &'a str) -> Vec<#ty> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::
            __find_all(
                #table_name,
                datasource_name
            ).await
                .ok()
                .unwrap()
                .get_entities::<#ty>()
        }
    }   
}

/// Generates the TokenStream for build the __find_all_result() CRUD 
/// associated function
pub fn generate_find_all_result_tokens(macro_data: &MacroTokens<'_>) -> TokenStream {
    let (vis, ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_struct(ty);

    quote! {
        /// Performns a `SELECT * FROM table_name`, where `table_name` it's
        /// the name of your entity but converted to the corresponding
        /// database convention. P.ej. PostgreSQL preferes table names declared
        /// with snake_case identifiers.
        #vis async fn find_all_result<'a>() -> 
            Result<Vec<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>> 
        {
            let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::
            __find_all(
                #table_name,
                ""
            ).await;

            if let Err(error) = result {
                Err(error)
            } else {
                Ok(result.ok().unwrap().get_entities::<#ty>())
            }
        }

        /// Performns a `SELECT * FROM table_name`, where `table_name` it's
        /// the name of your entity but converted to the corresponding
        /// database convention. P.ej. PostgreSQL preferes table names declared
        /// with snake_case identifiers.
        /// 
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the [`&str`] 
        /// passed as parameter.
        /// 
        /// Also, returns a [`Vec<T>, Error>`], wrapping a possible failure
        /// querying the database, or, if no errors happens, a Vec<T> containing
        /// the data found.
        #vis async fn find_all_result_datasource<'a>(datasource_name: &'a str) -> 
            Result<Vec<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>> 
        {
            let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::
            __find_all(
                #table_name,
                datasource_name
            ).await;

            if let Err(error) = result {
                Err(error)
            } else {
                Ok(result.ok().unwrap().get_entities::<#ty>())
            }
        }
    }
}

/// Same as above, but with a [`query_elements::query_builder::QueryBuilder`]
pub fn generate_find_all_query_tokens(macro_data: &MacroTokens<'_>) -> TokenStream {
    let (vis, ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_struct(ty);

    quote! {
        /// Generates a [`canyon_sql::canyon_crud::query_elements::query_builder::QueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        /// 
        /// It performs a `SELECT * FROM  table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention.
        #vis fn find_all_query<'a>() -> query_elements::query_builder::QueryBuilder<'a, #ty> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_all_query(
                #table_name, ""
            )
        }

        /// Generates a [`canyon_sql::canyon_crud::query_elements::query_builder::QueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        /// 
        /// It performs a `SELECT * FROM  table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention.
        /// 
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the [`&str`] 
        /// passed as parameter.
        #vis fn find_all_query_datasource<'a>(datasource_name: &'a str) -> 
            query_elements::query_builder::QueryBuilder<'a, #ty> 
        {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_all_query(
                #table_name, datasource_name
            )
        }
    }
}

/// Performs a COUNT(*) query over some table
pub fn generate_count_tokens(macro_data: &MacroTokens<'_>) -> TokenStream {
    let (vis, ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_struct(ty);

    quote! {
        /// TODO docs
        #vis async fn count() -> i64 {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__count(
                #table_name,
                ""
            ).await
            .ok()
            .unwrap()
        }

        /// TODO docs
        #vis async fn count_datasource<'a>(datasource_name: &'a str) -> i64 {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__count(
                #table_name,
                datasource_name
            ).await
            .ok()
            .unwrap()
        }
    }
}

/// Performs a COUNT(*) query over some table, returning a [`Result`] wrapping
/// a posible success or error coming from the database
pub fn generate_count_result_tokens(macro_data: &MacroTokens<'_>) -> TokenStream {
    let (vis, ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_struct(ty);

    quote! {
        /// TODO docs
        #vis async fn count_result() -> Result<i64, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__count(
                #table_name,
                ""
            ).await
        }

        /// TODO docs
        #vis async fn count_result_datasource<'a>(datasource_name: &'a str) -> Result<i64, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__count(
                #table_name,
                datasource_name
            ).await
        }
    }
}

/// Generates the TokenStream for build the __find_by_pk() CRUD operation
pub fn generate_find_by_pk_tokens(macro_data: &MacroTokens<'_>) -> TokenStream {
    let (vis, ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_struct(ty);

    let pk = macro_data.get_primary_key_annotation()
        .unwrap_or_default();

    // Disabled if there's no `primary_key` annotation
    if pk == "" { return quote! {}; }

    quote! {
        /// Finds an element on the queried table that matches the 
        /// value of the field annotated with the `primary_key` attribute, 
        /// filtering by the column that it's declared as the primary 
        /// key on the database.
        /// 
        /// This operation it's only available if the [`CanyonEntity`] contains
        /// a field declared as primary key.
        #vis async fn find_by_pk<'a>(
            pk_value: &'a dyn canyon_sql::canyon_crud::bounds::QueryParameters<'a>
        ) -> Option<#ty> {
            
            let response = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_by_pk(
                #table_name,
                #pk,
                pk_value,
                ""
            ).await;
                
            if response.as_ref().is_ok() {
                match response.as_ref().ok().unwrap() {
                    n if n.wrapper.len() == 0 => None,
                    _ => Some(
                        response
                            .ok()
                            .unwrap()
                            .get_entities::<#ty>()[0]
                            .clone()
                    )
                }
            } else { None }
        }

        /// Finds an element on the queried table that matches the 
        /// value of the field annotated with the `primary_key` attribute, 
        /// filtering by the column that it's declared as the primary 
        /// key on the database with the specified datasource.
        /// 
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the [`&str`] 
        /// passed as parameter.
        /// 
        /// This operation it's only available if the [`CanyonEntity`] contains
        /// a field declared as primary key.
        #vis async fn find_by_pk_datasource<'a>(
            pk_value: &'a dyn canyon_sql::canyon_crud::bounds::QueryParameters<'a>,
            datasource_name: &'a str
        ) -> Option<#ty> {
            /// TODO docs
            let response = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_by_pk(
                #table_name,
                #pk,
                pk_value,
                datasource_name
            ).await;
                
            if response.as_ref().is_ok() {
                match response.as_ref().ok().unwrap() {
                    n if n.wrapper.len() == 0 => None,
                    _ => Some(
                        response
                            .ok()
                            .unwrap()
                            .get_entities::<#ty>()[0]
                            .clone()
                    )
                }
            } else { None }
        }
    }
}

/// Generates the TokenStream for build the __find_by_pk() CRUD operation
pub fn generate_find_by_pk_result_tokens(macro_data: &MacroTokens<'_>) -> TokenStream {
    let (vis, ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_struct(ty);

    let pk = macro_data.get_primary_key_annotation()
        .unwrap_or_default();

    // Disabled if there's no `primary_key` annotation
    if pk == "" { return quote! {}; }

    quote! {
        /// Finds an element on the queried table that matches the 
        /// value of the field annotated with the `primary_key` attribute, 
        /// filtering by the column that it's declared as the primary 
        /// key on the database.
        /// 
        /// This operation it's only available if the [`CanyonEntity`] contains
        /// some field declared as primary key.
        /// 
        /// Also, returns a [`Result<Option<T>, Error>`], wrapping a possible failure
        /// querying the database, or, if no errors happens, a success containing
        /// and Option<T> with the data found wrapped in the Some(T) variant,
        /// or None if the value isn't found on the table.
        #vis async fn find_by_pk_result<'a>(pk_value: &'a dyn canyon_sql::canyon_crud::bounds::QueryParameters<'a>) -> 
            Result<Option<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
        {
            let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_by_pk(
                #table_name,
                #pk,
                pk_value,
                ""
            ).await;
                
            if let Err(error) = result {
                Err(error)
            } else { 
                match result.as_ref().ok().unwrap() {
                    n if n.wrapper.len() == 0 => Ok(None),
                    _ => Ok(Some(
                        result
                            .ok()
                            .unwrap()
                            .get_entities::<#ty>()[0]
                            .clone()
                    ))
                } 
            }
        }

        /// Finds an element on the queried table that matches the 
        /// value of the field annotated with the `primary_key` attribute, 
        /// filtering by the column that it's declared as the primary 
        /// key on the database.
        /// 
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the [`&str`] 
        /// passed as parameter.
        /// 
        /// This operation it's only available if the [`CanyonEntity`] contains
        /// some field declared as primary key.
        /// 
        /// Also, returns a [`Result<Option<T>, Error>`], wrapping a possible failure
        /// querying the database, or, if no errors happens, a success containing
        /// and Option<T> with the data found wrapped in the Some(T) variant,
        /// or None if the value isn't found on the table.
        #vis async fn find_by_pk_result_datasource<'a>(
            pk_value: &'a dyn canyon_sql::canyon_crud::bounds::QueryParameters<'a>,
            datasource_name: &'a str
        ) -> Result<Option<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
            let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_by_pk(
                #table_name,
                #pk, 
                pk_value,
                datasource_name
            ).await;
                
            if let Err(error) = result {
                Err(error)
            } else { 
                match result.as_ref().ok().unwrap() {
                    n if n.wrapper.len() == 0 => Ok(None),
                    _ => Ok(Some(
                        result
                            .ok()
                            .unwrap()
                            .get_entities::<#ty>()[0]
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
pub fn generate_find_by_foreign_key_tokens(macro_data: &MacroTokens<'_>) -> TokenStream {
    // let mut foreign_keys_tokens = Vec::new();
    let mut column_name = String::new();

    let current_entity = CANYON_REGISTER_ENTITIES.lock().unwrap();

    if let Some(entity) = &current_entity.iter()
        .find( |e| 
            *e.entity_name == database_table_name_from_entity_name(&macro_data.ty.to_string()) 
        ) 
    {
        for field in &entity.entity_fields {
            // Get the annotations attached to the entity, if some
            for annotation in &field.annotations {
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
                    let method_name = "search_".to_owned() + fk_table;
                    // Generate and identifier for the method based on the convention of "search_related_types" 
                    // where types is a placeholder for the plural name of the type referenced
                    let method_name_ident = proc_macro2::Ident::new(
                        &method_name, proc_macro2::Span::call_site()
                    );
                    let method_name_ident_ds = proc_macro2::Ident::new(
                        &format!("{}_datasource", &method_name), proc_macro2::Span::call_site()
                    );
                    let quoted_method_name: TokenStream = quote! { #method_name_ident }.into();
                    let quoted_method_name_ds: TokenStream = quote! { #method_name_ident_ds }.into();

                    // Converts a database table name generated by convection (lower case separated
                    // by underscores) to the Rust struct identifier convenction
                    let fk_ty = database_table_name_to_struct_ident(fk_table);

                    // The ident for the field that holds the FK relation
                    let field_ident = proc_macro2::Ident::new(
                        &field.field_name, proc_macro2::Span::call_site()
                    );

                    let field_value = quote! { &self.#field_ident };
                    
                    return quote! {
                        /// Searches the parent entity (if exists) for this type
                        pub async fn #quoted_method_name(&self) -> Option<#fk_ty> {
                            let lookage_value = #field_value.to_string();
                            let response = <#fk_ty as canyon_sql::canyon_crud::crud::CrudOperations<#fk_ty>>::
                                __search_by_foreign_key(#fk_table, #fk_column, &lookage_value, "")
                                    .await;
                            
                            if response.as_ref().is_ok() {
                                match response.as_ref().ok().unwrap() {
                                    n if n.wrapper.len() == 0 => None,
                                    _ => Some(
                                        response
                                            .ok()
                                            .unwrap()
                                            .get_entities::<#fk_ty>()[0]
                                            .clone()
                                    )
                                }
                            } else { None }
                        }

                        /// Searches the parent entity (if exists) for this type for the specified datasource
                        pub async fn #quoted_method_name_ds<'a>(&self, datasource_name: &'a str) -> Option<#fk_ty> {
                            let lookage_value = #field_value.to_string();
                            let response = <#fk_ty as canyon_sql::canyon_crud::crud::CrudOperations<#fk_ty>>::
                                __search_by_foreign_key(#fk_table, #fk_column, &lookage_value, datasource_name)
                                    .await;
                            
                            if response.as_ref().is_ok() {
                                match response.as_ref().ok().unwrap() {
                                    n if n.wrapper.len() == 0 => None,
                                    _ => Some(
                                        response
                                            .ok()
                                            .unwrap()
                                            .get_entities::<#fk_ty>()[0]
                                            .clone()
                                    )
                                }
                            } else { None }
                        }
                    };         
                }
            }
        }
    }

    quote! {}
}

/// Generates the TokenStream for build the search by foreign key feature, also as a method instance
/// of a T type of as an associated function of same T type
pub fn generate_find_by_foreign_key_result_tokens(macro_data: &MacroTokens<'_>) -> TokenStream {
    let mut column_name = String::new();
    let current_entity = CANYON_REGISTER_ENTITIES.lock().unwrap();

    if let Some(entity) = &current_entity.iter()
        .find( |e| 
            *e.entity_name == database_table_name_from_entity_name(&macro_data.ty.to_string()) 
        ) 
    {
        for field in &entity.entity_fields {
            // Get the annotations attached to the entity, if some
            for annotation in &field.annotations {
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
                    let method_name_ident_ds = proc_macro2::Ident::new(
                        &format!("{}_datasource", &method_name), proc_macro2::Span::call_site()
                    );
                    let quoted_method_name: TokenStream = quote! { #method_name_ident }.into();
                    let quoted_method_name_ds: TokenStream = quote! { #method_name_ident_ds }.into();

                    // Converts a database table name generated by convection (lower case separated
                    // by underscores) to the Rust struct identifier convenction
                    let fk_ty = database_table_name_to_struct_ident(fk_table);

                    // The ident for the field that holds the FK relation
                    let field_ident = proc_macro2::Ident::new(
                        &field.field_name, proc_macro2::Span::call_site()
                    );
                    let field_value = quote! { &self.#field_ident };

                    return quote! {
                        // Searches the parent entity (if exists) for this type
                        pub async fn #quoted_method_name(&self) -> 
                            Result<Option<#fk_ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>> 
                        {
                            let lookage_value = #field_value.to_string();
                            let result = <#fk_ty as canyon_sql::canyon_crud::crud::CrudOperations<#fk_ty>>::
                                __search_by_foreign_key(#fk_table, #fk_column, &lookage_value, "")
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
                                            .get_entities::<#fk_ty>()[0]
                                            .clone()
                                    ))
                                } 
                            }
                        }

                        pub async fn #quoted_method_name_ds<'a>(&self, datasource_name: &'a str) -> 
                            Result<Option<#fk_ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>> 
                        {
                            let lookage_value = #field_value.to_string();
                            let result = <#fk_ty as canyon_sql::canyon_crud::crud::CrudOperations<#fk_ty>>::
                                __search_by_foreign_key(#fk_table, #fk_column, &lookage_value, datasource_name)
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
                                            .get_entities::<#fk_ty>()[0]
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
                            Result<Option<#fk_ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>> 
                                where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
                        {
                            let lookage_value = value.get_fk_column(#fk_column).expect("Column not found");
                            let result = <#fk_ty as canyon_sql::canyon_crud::crud::CrudOperations<#fk_ty>>::
                                __search_by_foreign_key(#fk_table, #fk_column, &lookage_value, "")
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
                                            .get_entities::<#fk_ty>()[0]
                                            .clone()
                                    ))
                                } 
                            }
                        }

                        pub async fn belongs_to_result_datasource<'a, T>(value: &T, datasource_name: &'a str) ->
                            Result<Option<#fk_ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>> 
                                where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
                        {
                            let lookage_value = value.get_fk_column(#fk_column).expect("Column not found");
                            let result = <#fk_ty as canyon_sql::canyon_crud::crud::CrudOperations<#fk_ty>>::
                                __search_by_foreign_key(#fk_table, #fk_column, &lookage_value, datasource_name)
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
                                            .get_entities::<#fk_ty>()[0]
                                            .clone()
                                    ))
                                } 
                            }
                        }
                    }
                }
            }
        }
    }

    quote! {}
}

/// Generates the TokenStream for build the __search_by_foreign_key() CRUD 
/// associated function
pub fn generate_find_by_reverse_foreign_key_tokens(macro_data: &MacroTokens<'_>) -> Vec<TokenStream> {
    let mut foreign_keys_tokens = Vec::new();

    let (vis, ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_entity_name(&macro_data.ty.to_string());

    let mut column_name = String::new();
    let mut lookage_value_column = String::new();

    // Find what relation belongs to the data passed in
    let current_entity = CANYON_REGISTER_ENTITIES.lock().unwrap();

    if let Some(entity) = &current_entity.iter()
        .find( |e| 
            e.entity_name == &table_name
        ) 
    {
        for field in &entity.entity_fields {
            // Get the annotations
            for annotation in &field.annotations {
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
                    let method_name_ident_ds = proc_macro2::Ident::new(
                        &format!("{}_datasource", &method_name), proc_macro2::Span::call_site()
                    );
                    let quoted_method_name: TokenStream = quote! { #method_name_ident }.into();
                    let quoted_method_name_ds: TokenStream = quote! { #method_name_ident_ds }.into();

                    foreign_keys_tokens.push(
                        quote! {
                            #vis async fn #quoted_method_name<T>(value: &T) -> Vec<#ty> 
                                where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
                            {
                                let lookage_value = value.get_fk_column(#lookage_value_column).expect("Column not found");
                                <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::
                                    __search_by_reverse_side_foreign_key(#table_name, #column_name, lookage_value, "")
                                        .await
                                        .ok()
                                        .expect("Error looking for the related entities on the FK reverse search")
                                        .get_entities::<#ty>()
                            }

                            #vis async fn #quoted_method_name_ds<'a, T>(value: &T, datasource_name: &'a str) -> Vec<#ty> 
                                where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
                            {
                                let lookage_value = value.get_fk_column(#lookage_value_column).expect("Column not found");
                                <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::
                                    __search_by_reverse_side_foreign_key(#table_name, #column_name, lookage_value, datasource_name)
                                        .await
                                        .ok()
                                        .expect("Error looking for the related entities on the FK reverse search ds")
                                        .get_entities::<#ty>()
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
pub fn generate_find_by_reverse_foreign_key_result_tokens(macro_data: &MacroTokens<'_>) -> Vec<TokenStream> {
    let mut foreign_keys_tokens = Vec::new();

    let (vis, ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_entity_name(&macro_data.ty.to_string());

    let mut column_name = String::new();
    let mut lookage_value_column = String::new();

    // Find what relation belongs to the data passed in
    let current_entity = CANYON_REGISTER_ENTITIES.lock().unwrap();

    if let Some(entity) = &current_entity.iter()
        .find( |e| 
            e.entity_name == &table_name
        ) 
    {
        for field in &entity.entity_fields {
            // Get the annotations
            for annotation in &field.annotations {
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
                    let method_name_ident_ds = proc_macro2::Ident::new(
                        &format!("{}_datasource", &method_name), proc_macro2::Span::call_site()
                    );
                    let quoted_method_name: TokenStream = quote! { #method_name_ident }.into();
                    let quoted_method_name_ds: TokenStream = quote! { #method_name_ident_ds }.into();

                    foreign_keys_tokens.push(
                        quote! {
                            #vis async fn #quoted_method_name<T>(value: &T) -> 
                                Result<Vec<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>> 
                                    where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
                            {
                                let lookage_value = value.get_fk_column(#lookage_value_column).expect("Column not found");
                                let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::
                                    __search_by_reverse_side_foreign_key(#table_name, #column_name, lookage_value, "")
                                        .await;

                                if let Err(error) = result {
                                    Err(error)
                                } else {
                                    Ok(result.ok().unwrap().get_entities::<#ty>())
                                }
                            }

                            #vis async fn #quoted_method_name_ds<'a, T>(value: &'a T, datasource_name: &'a str) -> 
                                Result<Vec<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>> 
                                    where T: canyon_sql::canyon_crud::bounds::ForeignKeyable 
                            {
                                let lookage_value = value.get_fk_column(#lookage_value_column).expect("Column not found");
                                let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::
                                    __search_by_reverse_side_foreign_key(#table_name, #column_name, lookage_value, datasource_name)
                                        .await;

                                if let Err(error) = result {
                                    Err(error)
                                } else {
                                    Ok(result.ok().unwrap().get_entities::<#ty>())
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

/// Helper to get the plural noun of a given table identifier
fn _make_related_table_plural(singular: &str) -> String {
    // TODO Generate the correct forms of the plural for a given identifier
    // For brevity, and for now, just adds an 's' to the end of the noun
    singular.to_owned() + "s"
}