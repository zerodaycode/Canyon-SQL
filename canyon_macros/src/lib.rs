extern crate proc_macro;

use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    DeriveInput, Fields, Visibility, parse_macro_input, ItemFn
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
use canyon_macro::{_user_body_builder, wire_queries_to_execute};
use canyon_observer::{
     handler::{CanyonHandler, CanyonRegisterEntity, CanyonRegisterEntityField}, CANYON_REGISTER_ENTITIES,
};


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

    // The code used by Canyon to perform it's managed state
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        CanyonHandler::run().await;
    });

    // The queries to execute at runtime in the managed state
    let mut queries_tokens: Vec<TokenStream> = Vec::new();
    wire_queries_to_execute(&mut queries_tokens);
    
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
                #(#queries_tokens)*
            }
            #(#macro_tokens)*
        }
    };
    
    tokens.into()
}



/// Takes data from the struct annotated with macro to fill the Canyon Register
/// where lives the data that Canyon needs to work in `managed mode`
#[proc_macro_attribute]
pub fn canyon_entity(_meta: CompilerTokenStream, input: CompilerTokenStream) -> CompilerTokenStream {
    let entity = syn::parse_macro_input!(input as CanyonEntity);

    // Generate the bits of code that we should give back to the compiler
    let generated_data_struct = generate_data_struct(&entity);  
    get_field_attr(&entity);

    // Notifies the observer that an observable must be registered on the system
    // In other words, adds the data of the structure to the Canyon Register
    println!("Observable of new register <{}> added to the register", &entity.struct_name.to_string());
    let mut new_entity = CanyonRegisterEntity::new();
    new_entity.entity_name = entity.struct_name.to_string().to_lowercase();
    for field in entity.attributes.iter() {
        let mut new_entity_field = CanyonRegisterEntityField::new();
        new_entity_field.field_name = field.name.to_string();
        new_entity_field.field_type = field.get_field_type_as_string().replace(" ", "");
        new_entity.entity_fields.push(new_entity_field);
    }
    unsafe { CANYON_REGISTER_ENTITIES.push(new_entity) }

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
        impl canyon_crud::crud::CrudOperations<#ty> for #ty { }
        impl canyon_crud::crud::Transaction<#ty> for #ty { }
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

        impl canyon_sql::canyon_crud::mapper::RowMapper<Self> for #ty {
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
