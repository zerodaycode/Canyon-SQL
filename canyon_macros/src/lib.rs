extern crate proc_macro;

use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    DeriveInput, Fields, Visibility, parse_macro_input, ItemFn, Type
};


mod canyon_macro;
mod query_operations;
mod utils;

use utils::macro_tokens::MacroTokens;
use query_operations::{
    insert::generate_insert_tokens, 
    select::{generate_find_all_tokens, generate_find_by_id_tokens},
    delete::generate_delete_tokens,
    update::generate_update_tokens
};
use canyon_manager::manager::{
    manager_builder::{
        generate_data_struct, 
        get_field_attr
    }, 
    entity::CanyonEntity
};
use canyon_macro::{
    _user_body_builder, 
    _wire_data_on_canyon_register, 
    call_canyon_manager
};
use canyon_observer::CANYON_REGISTER;


/// Macro for handling the entry point to the program. 
/// 
/// Avoids the user to write the tokio attribute and
/// the async modifier to the main fn()
/// 
/// Also, takes care about wire the necessary code for Canyon in order to 
/// full achieve it's complete set of features
/// TODO Check for the _meta attribute metadata when necessary
#[proc_macro_attribute]
pub fn canyon(_meta: CompilerTokenStream, input: CompilerTokenStream) -> CompilerTokenStream {
    // Get the function that this attribute is attached to
    let func = parse_macro_input!(input as ItemFn);
    let sign = func.sig;
    let body = func.block;

    // The code wired in main() by Canyon, but in it's own scope
    let mut canyon_manager_tokens: Vec<TokenStream> = Vec::new();
    // Builds the code that Canyon needs in it's initialization
    _wire_data_on_canyon_register(&mut canyon_manager_tokens);
    // Builds the code that Canyon uses to manage the ORM
    call_canyon_manager(&mut canyon_manager_tokens);

    // The code written by the user
    let mut macro_tokens: Vec<TokenStream> = Vec::new();
    // Builds the code that represents the user written code
    _user_body_builder(body, &mut macro_tokens);
    

    // The final code wired in main()
    let tokens = quote! {
        use canyon_sql::tokio;
        #[tokio::main]
        async #sign {
            {     
                use canyon_sql::{
                    canyon_observer::CANYON_REGISTER,
                    handler::CanyonHandler
                };
                use canyon_sql::tokio_postgres::types::Type;
                use std::collections::HashMap;

                #(#canyon_manager_tokens)*
            }

            #(#macro_tokens)*
        }
    };
    
    tokens.into()
}


/// Takes data from the struct annotated with macro to fill the Canyon Register
/// where lives the data that Canyon needs to work in `managed mode`
#[proc_macro_attribute]
pub fn canyon_managed(_meta: CompilerTokenStream, input: CompilerTokenStream) -> CompilerTokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (vis, ty, generics) = (&ast.vis, &ast.ident, &ast.generics);
    let fields = fields_with_types(
        match ast.data {
            syn::Data::Struct(ref s) => &s.fields,
            _ => panic!("Field names can only be derived for structs"),
        }
    );

    // Notifies the observer that an observable must be registered on the system
    // In other words, adds the data of the structure to the Canyon Register
    // unsafe { CANYON_REGISTER.push(ty.to_string()); }
    println!("Observable <{}> added to the register", ty.to_string());

    
    let struct_fields = fields.iter().map(|(_vis, ident, ty)| {
        quote! {  
            #vis #ident: #ty
        }
    });

    let (_impl_generics, ty_generics, _where_clause) = 
        generics.split_for_impl();

    let tokens = quote! {
        pub struct #ty <#ty_generics> {
            #(#struct_fields),*
        }
    };

    tokens.into()
}

/// TODO Docs
#[proc_macro_attribute]
pub fn canyon_entity(_meta: CompilerTokenStream, input: CompilerTokenStream) -> CompilerTokenStream {
    let entity = syn::parse_macro_input!(input as CanyonEntity);

    // Generate the bits of code that we should give back to the compiler
    let generated_data_struct = generate_data_struct(&entity);  
    get_field_attr(&entity);

    // Notifies the observer that an observable must be registered on the system
    // In other words, adds the data of the structure to the Canyon Register
    println!("Observable of new register <{}> added to the register", &entity.struct_name.to_string());
    unsafe { CANYON_REGISTER.push(entity.get_entity_as_string()) }

    // Assemble everything
    let tokens = quote! {
        #generated_data_struct
    };

    // Pass the result back to the compiler
    tokens.into()
}

/// Allows the implementors to auto-derive de `crud-operations` trait, which defines the methods
/// that will perform the database communication and that will query against the db.
#[proc_macro_derive(CanyonCRUD)]
pub fn crud_operations(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_crud_operations_trait_for_struct(&ast)
}


fn impl_crud_operations_trait_for_struct(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &ast.ident;
    let tokens = quote! {
        #[async_trait]
        impl canyon_sql::crud::CrudOperations<#ty> for #ty { }
        impl canyon_sql::crud::Transaction<#ty> for #ty { }
    };
    tokens.into()
}


#[proc_macro_derive(CanyonMapper)]
pub fn implement_row_mapper_for_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Gets the data from the AST
    let ast: DeriveInput = syn::parse(input).unwrap();

    // Constructs a new instance of the helper that manages the macro data
    let macro_data = MacroTokens::new(&ast);

    // Builds the find_all() query
    let find_all_tokens = generate_find_all_tokens(&macro_data);
    // Builds the find_by_id() query
    let find_by_id_tokens = generate_find_by_id_tokens(&macro_data);
    // Builds the insert() query
    let insert_tokens = generate_insert_tokens(&macro_data);
    // Builds the delete() query
    let delete_tokens = generate_delete_tokens(&macro_data);
    // Builds the update() query
    let update_tokens = generate_update_tokens(&macro_data);

    // Recoves the identifiers of the struct's members
    let fields = filter_fields(
        match ast.data {
            syn::Data::Struct(ref s) => &s.fields,
            _ => panic!("Field names can only be derived for structs"),
        }
    );

    // Creates the TokenStream for wire the column names into the 
    // Canyon RowMapper
    let field_names_for_row_mapper = fields.iter().map(|(_vis, ident)| {
        let ident_name = ident.to_string();
        quote! {  
            #ident: row.try_get(#ident_name)
                .expect(format!("Failed to retrieve the {} field", #ident_name).as_ref())
        }
    });

    // The type of the Struct
    let ty = macro_data.ty;
    
    // Get the generics identifiers
    let (impl_generics, ty_generics, where_clause) = 
        macro_data.generics.split_for_impl();


    let tokens = quote! {
        impl #impl_generics #ty #ty_generics
            #where_clause
        {
            // The find_by_id impl
            #find_all_tokens

            // The find_by_id impl
            #find_by_id_tokens

            // The insert impl
            #insert_tokens

            // The delete impl
            #delete_tokens

            // The update impl
            #update_tokens

        }

        impl RowMapper<Self> for #ty {
            fn deserialize(row: &Row) -> Self {
                Self {
                    #(#field_names_for_row_mapper),*
                }
            }
        }
    };

    tokens.into()
}


fn filter_fields(fields: &Fields) -> Vec<(Visibility, Ident)> {
    fields
        .iter()
        .map(|field| 
            (field.vis.clone(), field.ident.as_ref().unwrap().clone()) 
        )
        .collect::<Vec<_>>()
}


fn fields_with_types(fields: &Fields) -> Vec<(Visibility, Ident, Type)> {
    fields
        .iter()
        .map(|field| 
            (field.vis.clone(), 
            field.ident.as_ref().unwrap().clone(),
            field.ty.clone()
        ) 
        )
        .collect::<Vec<_>>()
}