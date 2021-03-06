extern crate proc_macro;

mod canyon_macro;
mod query_operations;
mod utils;

use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    DeriveInput, Fields, Visibility
};

use query_operations::{
    select::{
        generate_find_all_tokens,
        generate_find_all_result_tokens,
        generate_find_all_query_tokens,
        generate_count_tokens,
        generate_count_result_tokens,
        generate_find_by_id_tokens,
        generate_find_by_id_result_tokens,
        generate_find_by_foreign_key_tokens,
        generate_find_by_foreign_key_result_tokens,
        generate_find_by_reverse_foreign_key_tokens,
        generate_find_by_reverse_foreign_key_result_tokens
    },
    insert::{
        generate_insert_tokens,
        generate_insert_result_tokens,
        generate_multiple_insert_tokens
    }, 
    update::{
        generate_update_tokens,
        generate_update_result_tokens,
        generate_update_query_tokens
    },
    delete::{
        generate_delete_tokens,
        generate_delete_result_tokens,
        generate_delete_query_tokens
    }
};

use utils::{
    function_parser::FunctionParser,
    macro_tokens::MacroTokens 
};
use canyon_macro::wire_queries_to_execute;

use canyon_manager::manager::{
    manager_builder::{
        generate_user_struct,
        generate_enum_with_fields,
        generate_enum_with_fields_values 
    }, 
    entity::CanyonEntity
};

use canyon_observer::{
    CANYON_REGISTER_ENTITIES,
    handler::CanyonHandler, 
    postgresql::register_types::{
        CanyonRegisterEntity, 
        CanyonRegisterEntityField
    }, 
};


/// Macro for handling the entry point to the program. 
/// 
/// Avoids the user to write the tokio proc_attribute and
/// the async modifier to the main fn()
/// 
/// Also, takes care about wire the necessary code that Canyon's need
/// to run in order to check the provided code and in order to perform
/// the necessary operations for the migrations
#[proc_macro_attribute]
pub fn canyon(_meta: CompilerTokenStream, input: CompilerTokenStream) -> CompilerTokenStream {
    // Parses the function that this attribute is attached to
    let func_res = syn::parse::<FunctionParser>(input);
    
    if func_res.is_err() {
        return quote! { fn main() {} }.into()
    }
    
    let func = func_res.ok().unwrap();
    let sign = func.clone().sig;
    let body = func.clone().block.stmts;

    // The code used by Canyon to perform it's managed state
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        CanyonHandler::run().await;
    });

    // The queries to execute at runtime in the managed state
    let mut queries_tokens: Vec<TokenStream> = Vec::new();
    wire_queries_to_execute(&mut queries_tokens);
    
    // The final code wired in main()
    quote! {
        use canyon_sql::tokio;
        #[tokio::main]
        async #sign {
            {
                #(#queries_tokens)*
            }
            #(#body)*
        }
    }.into()

}


/// Takes data from the struct annotated with macro to fill the Canyon Register
/// where lives the data that Canyon needs to work in `managed mode`
#[proc_macro_attribute]
pub fn canyon_entity(_meta: CompilerTokenStream, input: CompilerTokenStream) -> CompilerTokenStream {
    let input_cloned = input.clone();
    let entity_res = syn::parse::<CanyonEntity>(input);
    
    if entity_res.is_err() {
        return entity_res.err().unwrap().into_compile_error().into()
    }

    // No errors detected on the parsing, so we can safely unwrap the parse result
    let entity = entity_res.ok().unwrap();

    // Generate the bits of code that we should give back to the compiler
    let generated_user_struct = generate_user_struct(&entity);
    let generated_enum_type_for_fields = generate_enum_with_fields(&entity);
    let generated_enum_type_for_fields_values = generate_enum_with_fields_values(&entity);

    // The identifier of the entities
    let mut new_entity = CanyonRegisterEntity::new();
    new_entity.entity_name = entity.struct_name.to_string().to_lowercase();

    // The entity fields
    for field in entity.attributes.iter() {
        let mut new_entity_field = CanyonRegisterEntityField::new();
        new_entity_field.field_name = field.name.to_string();
        new_entity_field.field_type = field.get_field_type_as_string().replace(" ", "");
        
        // The annotations
        if let Some(annotation) = &field.attribute {
            new_entity_field.annotation = Some(annotation.get_as_string())
        }

        new_entity.entity_fields.push(new_entity_field);
    }

    // Fill the register with the data of the attached struct
    (*CANYON_REGISTER_ENTITIES).lock().unwrap().push(new_entity);

    // Struct name as Ident for wire in the macro
    let ty = entity.struct_name;

    // Calls the helper struct to build the tokens that generates the final CRUD methos
    let ast: DeriveInput = syn::parse(input_cloned).unwrap();
    let macro_data = MacroTokens::new(&ast);

    // Builds the find_all() query
    let find_all_tokens = generate_find_all_tokens(&macro_data);
    // Builds the find_all_result() query
    let find_all_result_tokens = generate_find_all_result_tokens(&macro_data);
    // Builds the find_all_query() query as a QueryBuilder
    let find_all_query_tokens = generate_find_all_query_tokens(&macro_data);
    
    // Builds a COUNT(*) query over some table
    let count_tokens = generate_count_tokens(&macro_data);
    // Builds a COUNT(*) query over some table
    let count_result_tokens = generate_count_result_tokens(&macro_data);
   
    // Builds the find_by_id() query
    let find_by_id_tokens = generate_find_by_id_tokens(&macro_data);
    // Builds the find_by_id_result() query
    let find_by_id_result_tokens = generate_find_by_id_result_tokens(&macro_data);
    
    // Builds the insert() query
    let insert_tokens = generate_insert_tokens(&macro_data);
    // Builds the insert() query as a result
    let insert_result_tokens = generate_insert_result_tokens(&macro_data);
    // Builds the insert_multi() query
    let insert_multi_tokens = generate_multiple_insert_tokens(&macro_data);
    
    // Builds the update() query
    let update_tokens = generate_update_tokens(&macro_data);
    // Builds the update() query as a result
    let update_result_tokens = generate_update_result_tokens(&macro_data);
    // Builds the update() query as a QueryBuilder
    let update_query_tokens = generate_update_query_tokens(&macro_data);

    // Builds the delete() query
    let delete_tokens = generate_delete_tokens(&macro_data);
    // Builds the delete() query as a result
    let delete_result_tokens = generate_delete_result_tokens(&macro_data);
    // Builds the delete() query as a QueryBuilder
    let delete_query_tokens = generate_delete_query_tokens(&macro_data);
    

    // Search by foreign (d) key as Vec, cause Canyon supports multiple fields having FK annotation
    let search_by_fk_tokens: Vec<TokenStream> = generate_find_by_foreign_key_tokens();
    let search_by_fk_result_tokens: Vec<TokenStream> = generate_find_by_foreign_key_result_tokens();
    let search_by_revese_fk_tokens: Vec<TokenStream> = generate_find_by_reverse_foreign_key_tokens(&macro_data);
    let search_by_revese_fk_result_tokens: Vec<TokenStream> = generate_find_by_reverse_foreign_key_result_tokens(&macro_data);

    // Get the generics identifiers
    let (impl_generics, ty_generics, where_clause) = 
    macro_data.generics.split_for_impl();
        
    // Assemble everything
    let tokens = quote! {
        #generated_user_struct

        impl #impl_generics #ty #ty_generics
            #where_clause
        {
            // The find_all impl
            #find_all_tokens

            // The find_all_result impl
            #find_all_result_tokens

            // The find_all_query impl
            #find_all_query_tokens

            // The COUNT(*) impl
            #count_tokens

            // The COUNT(*) as result impl
            #count_result_tokens

            // The find_by_id impl
            #find_by_id_tokens

            // The find_by_id as result impl
            #find_by_id_result_tokens

            // The insert impl
            #insert_tokens

            // The insert as a result impl
            #insert_result_tokens

            // The insert of multiple entities impl
            #insert_multi_tokens

            // The update impl
            #update_tokens

            // The update as result impl
            #update_result_tokens
            
            // The update as a querybuilder impl
            #update_query_tokens
            
            // The delete impl
            #delete_tokens

            // The delete as result impl
            #delete_result_tokens

            // The delete as querybuilder impl
            #delete_query_tokens

            // The search by FK impl
            #(#search_by_fk_tokens),*
            // The search by FK as result impl
            #(#search_by_fk_result_tokens),*

            // The search by reverse side of the FK impl
            #(#search_by_revese_fk_tokens),*
            // The search by reverse side of the FK as result impl
            #(#search_by_revese_fk_result_tokens),*
        }

        #generated_enum_type_for_fields

        #generated_enum_type_for_fields_values
    };

    // Pass the result back to the compiler
    tokens.into()
}

/// Allows the implementors to auto-derive de `crud-operations` trait, which defines the methods
/// that will perform the database communication and that will query against the db.
#[proc_macro_derive(CanyonCrud)]
pub fn crud_operations(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast: DeriveInput = syn::parse(input).unwrap();
    

    // Checks that this macro is on a struct
    match ast.data {
        syn::Data::Struct(ref _s) => (),
        _ => return syn::Error::new(
            ast.ident.span(), 
            "CanyonCrud only works with Structs"
        ).to_compile_error().into()
    }

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

/// proc-macro for annotate struct fields that holds a foreign key relation.
/// 
/// So basically, if you have some `ForeignKey` attribute, annotate the parent
/// struct (where the ForeignKey table property points) with this macro
/// to make it able to work with compound table relations
#[proc_macro_derive(ForeignKeyable)]
pub fn implement_foreignkeyable_for_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Gets the data from the AST
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ty = ast.ident;

    // Recovers the identifiers of the struct's members
    let fields = filter_fields(
        match ast.data {
            syn::Data::Struct(ref s) => &s.fields,
            _ => return syn::Error::new(
                ty.span(), 
                "ForeignKeyable only works with Structs"
            ).to_compile_error().into()
        }
    );

    let field_idents = fields.iter()
        .map( |(_vis, ident)|
            {
                let i = ident.to_string();
                quote! {
                    #i => Some(self.#ident.to_string())
                }
            }
    );
    
    quote!{
        impl canyon_sql::canyon_crud::bounds::ForeignKeyable for #ty {
            fn get_fk_column(&self, column: &str) -> Option<String> {
                match column {
                    #(#field_idents),*,
                    _ => None
                }
            }
        }
    }.into()
}

#[proc_macro_derive(CanyonMapper)]
pub fn implement_row_mapper_for_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Gets the data from the AST
    let ast: DeriveInput = syn::parse(input).unwrap();

    // Recovers the identifiers of the struct's members
    let fields = filter_fields(
        match ast.data {
            syn::Data::Struct(ref s) => &s.fields,
            _ => return syn::Error::new(
                ast.ident.span(), 
                "CanyonMapper only works with Structs"
            ).to_compile_error().into(),
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
    let ty = ast.ident;

    let tokens = quote! {
        impl canyon_sql::canyon_crud::mapper::RowMapper<Self> for #ty
        {
            fn deserialize(row: &Row) -> #ty {
                Self {
                    #(#field_names_for_row_mapper),*
                }
            }
        }
    };

    tokens.into()
}

/// Helper for generate the fields data for the Custom Derives Macros
fn filter_fields(fields: &Fields) -> Vec<(Visibility, Ident)> {
    fields
        .iter()
        .map(|field| 
            (field.vis.clone(), field.ident.as_ref().unwrap().clone()) 
        )
        .collect::<Vec<_>>()
}
