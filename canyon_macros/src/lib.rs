// TODO: remember to remove this allows
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

extern crate proc_macro;
extern crate regex;

mod canyon_entity_macro;
#[cfg(feature = "migrations")]
use canyon_macro::main_with_queries;

mod canyon_macro;
mod query_operations;
mod utils;
mod canyon_mapper_macro;

use canyon_entity_macro::parse_canyon_entity_proc_macro_attr;
use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{DeriveInput, Fields, Type, Visibility};

use query_operations::{
    read::generate_read_operations_tokens,
    insert::generate_insert_tokens,
    update::generate_update_tokens,
    delete::generate_delete_tokens,
    foreign_key::generate_find_by_fk_ops
};
use utils::{function_parser::FunctionParser, helpers, macro_tokens::MacroTokens};

use canyon_entities::{
    entity::CanyonEntity,
    manager_builder::{
        generate_enum_with_fields, generate_enum_with_fields_values, generate_user_struct,
    },
    register_types::{CanyonRegisterEntity, CanyonRegisterEntityField},
    CANYON_REGISTER_ENTITIES,
};
use crate::canyon_mapper_macro::canyon_mapper_impl_tokens;
use crate::utils::helpers::filter_fields;

/// Macro for handling the entry point to the program.
///
/// Avoids the user to write the tokio proc_attribute and
/// the async modifier to the main fn()
///
/// Also, takes care about wire the necessary code that Canyon's need
/// to run in order to check the provided code and in order to perform
/// the necessary operations for the migrations
#[proc_macro_attribute]
pub fn main(_meta: CompilerTokenStream, input: CompilerTokenStream) -> CompilerTokenStream {
    let func_res = syn::parse::<FunctionParser>(input);
    if func_res.is_err() {
        return quote! { fn main() {} }.into();
    }

    // TODO check if the `canyon` macro it's attached only to main?
    let func = func_res.ok().unwrap();
    let sign = func.sig;
    let body = func.block.stmts;

    #[allow(unused_mut, unused_assignments)]
    let mut migrations_tokens = quote! {};
    #[cfg(feature = "migrations")]
    {
        migrations_tokens = main_with_queries();
    }

    // The final code wired in main()
    quote! {
        #sign {
            canyon_sql::runtime::CANYON_TOKIO_RUNTIME
                .handle()
                .block_on( async {
                    canyon_sql::runtime::init_connections_cache().await;
                    #migrations_tokens
                    #(#body)*
                }
            )
        }
    }
    .into()
}

#[proc_macro_attribute]
/// Wraps the [`test`] proc macro in a convenient way to run tests within
/// the tokio's current reactor
pub fn canyon_tokio_test(
    _meta: CompilerTokenStream,
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    let func_res = syn::parse::<FunctionParser>(input);
    if func_res.is_err() {
        quote! { fn non_valid_test_fn() {} }.into()
    } else {
        let func = func_res.ok().unwrap();
        let sign = func.sig;
        let body = func.block.stmts;
        let attrs = func.attrs;

        quote! {
            #[test]
            #(#attrs)*
            #sign {
                canyon_sql::runtime::CANYON_TOKIO_RUNTIME
                    .handle()
                    .block_on( async {
                        canyon_sql::runtime::init_connections_cache().await;
                        #(#body)*
                    });
            }
        }
        .into()
    }
}

/// Generates the enums that contains the `TypeFields` and `TypeFieldsValues`
/// that the query-builder requires for construct its queries
#[proc_macro_derive(Fields)]
pub fn querybuilder_fields(input: CompilerTokenStream) -> CompilerTokenStream {
    let entity_res = syn::parse::<CanyonEntity>(input);

    if entity_res.is_err() {
        return entity_res
            .expect_err("Unexpected error parsing the struct")
            .into_compile_error()
            .into();
    }

    // No errors detected on the parsing, so we can safely unwrap the parse result
    let entity = entity_res.expect("Unexpected error parsing the struct");
    let _generated_enum_type_for_fields = generate_enum_with_fields(&entity);
    let _generated_enum_type_for_fields_values = generate_enum_with_fields_values(&entity);
    quote! {
        use canyon_sql::core::QueryParameter;
        #_generated_enum_type_for_fields
        #_generated_enum_type_for_fields_values
    }
    .into()
}

/// Takes data from the struct annotated with the `canyon_entity` macro to fill the Canyon Register
/// where lives the data that Canyon needs to work.
///
/// Also, it's the responsible of generate the tokens for all the `Crud` methods available over
/// your type
#[proc_macro_attribute]
pub fn canyon_entity(
    _meta: CompilerTokenStream,
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    let attrs = syn::parse_macro_input!(_meta as syn::AttributeArgs);

    let (table_name, schema_name, parsing_attribute_error) =
        parse_canyon_entity_proc_macro_attr(attrs);

    let entity_res = syn::parse::<CanyonEntity>(input);

    if entity_res.is_err() {
        return entity_res
            .expect_err("Unexpected error parsing the struct")
            .into_compile_error()
            .into();
    }

    // No errors detected on the parsing, so we can safely unwrap the parse result
    let entity = entity_res.unwrap();
    let generated_user_struct = generate_user_struct(&entity);

    // The identifier of the entities
    let mut new_entity = CanyonRegisterEntity::default();
    let e = Box::leak(entity.struct_name.to_string().into_boxed_str());
    new_entity.entity_name = e;
    new_entity.entity_db_table_name = table_name.unwrap_or(Box::leak(
        helpers::default_database_table_name_from_entity_name(e).into_boxed_str(),
    ));
    new_entity.user_schema_name = schema_name;

    // The entity fields
    for field in entity.fields.iter() {
        let mut new_entity_field = CanyonRegisterEntityField {
            field_name: field.name.to_string(),
            field_type: field.get_field_type_as_string().replace(' ', ""),
            ..Default::default()
        };

        field
            .attributes
            .iter()
            .for_each(|attr| new_entity_field.annotations.push(attr.get_as_string()));

        new_entity.entity_fields.push(new_entity_field);
    }

    // Fill the register with the data of the attached struct
    CANYON_REGISTER_ENTITIES
        .lock()
        .expect("Error acquiring Mutex guard on Canyon Entity macro")
        .push(new_entity);

    // Assemble everything
    let tokens = quote! {
        #generated_user_struct
    };

    // Pass the result back to the compiler
    if let Some(macro_error) = parsing_attribute_error {
        quote! {
            #macro_error
            #generated_user_struct
        }
        .into()
    } else {
        tokens.into()
    }
}

/// Allows the implementors to auto-derive the `CrudOperations` trait, which defines the methods
/// that will perform the database communication and the implementation of the queries for every
/// type, as defined in the `CrudOperations` + `Transaction` traits.
#[proc_macro_derive(CanyonCrud)]
pub fn crud_operations(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput =
        syn::parse(input).expect("Error parsing `Canyon Entity for generate the CRUD methods");
    let macro_data = MacroTokens::new(&ast);

    let table_name_res = helpers::table_schema_parser(&macro_data);

    let table_schema_data = if let Err(err) = table_name_res {
        return err.into();
    } else {
        table_name_res.ok().unwrap()
    };

    // Build the trait implementation
    impl_crud_operations_trait_for_struct(&macro_data, table_schema_data)
}

fn impl_crud_operations_trait_for_struct(
    macro_data: &MacroTokens<'_>,
    table_schema_data: String,
) -> proc_macro::TokenStream {
    let mut crud_ops_tokens = TokenStream::new();
    let ty = macro_data.ty;

    let read_operations_tokens = generate_read_operations_tokens(macro_data, &table_schema_data);
    let insert_tokens = generate_insert_tokens(macro_data, &table_schema_data);
    let update_tokens = generate_update_tokens(macro_data, &table_schema_data);
    let delete_tokens = generate_delete_tokens(macro_data, &table_schema_data);

    let crud_operations_tokens = quote! { // TODO: bring this directly from mod.rs or query_operations?
        #read_operations_tokens
        #insert_tokens
        #update_tokens
        #delete_tokens
    };

    crud_ops_tokens.extend(quote!{
        use canyon_sql::core::IntoResults;

        #[canyon_sql::macros::async_trait] // TODO: get rid of the async_trait
        impl canyon_sql::crud::CrudOperations<#ty> for #ty {
            #crud_operations_tokens
        }

        impl canyon_sql::core::Transaction<#ty> for #ty {}
    });
    
    let foreign_key_ops_tokens = generate_find_by_fk_ops(macro_data, &table_schema_data);
    crud_ops_tokens.extend(quote!{ #foreign_key_ops_tokens });
    
    crud_ops_tokens.into()
}

/// proc-macro for annotate struct fields that holds a foreign key relation.
///
/// So basically, if you have some `ForeignKey` attribute, annotate the parent
/// struct (where the ForeignKey table property points) with this macro
/// to make it able to work with compound table relations
#[proc_macro_derive(ForeignKeyable)]
pub fn implement_foreignkeyable_for_type(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Gets the data from the AST
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ty = ast.ident;

    // Recovers the identifiers of the structs members
    let fields = filter_fields(match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => {
            return syn::Error::new(ty.span(), "ForeignKeyable only works with Structs")
                .to_compile_error()
                .into()
        }
    });

    let field_idents = fields.iter().map(|(_vis, ident)| {
        let i = ident.to_string();
        quote! {
            #i => Some(&self.#ident as &dyn canyon_sql::core::QueryParameter<'_>)
        }
    });
    let field_idents_cloned = field_idents.clone();

    quote! {
        /// Implementation of the trait `ForeignKeyable` for the type
        /// calling this derive proc macro
        impl canyon_sql::crud::bounds::ForeignKeyable<Self> for #ty {
            fn get_fk_column(&self, column: &str) -> Option<&dyn canyon_sql::core::QueryParameter<'_>> {
                match column {
                    #(#field_idents),*,
                    _ => None
                }
            }
        }
        /// Implementation of the trait `ForeignKeyable` for a reference of this type
        /// calling this derive proc macro
        impl canyon_sql::crud::bounds::ForeignKeyable<&Self> for &#ty {
            fn get_fk_column<'a>(&self, column: &'a str) -> Option<&dyn canyon_sql::core::QueryParameter<'_>> {
                match column {
                    #(#field_idents_cloned),*,
                    _ => None
                }
            }
        }
    }.into()
}

#[proc_macro_derive(CanyonMapper)]
pub fn implement_row_mapper_for_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    canyon_mapper_impl_tokens(ast).into()
}

