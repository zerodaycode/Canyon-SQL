extern crate proc_macro;
extern crate regex;

#[cfg(feature = "migrations")]
use canyon_macro::main_with_queries;

mod canyon_entity_macro;
mod canyon_macro;
mod canyon_mapper_macro;
mod foreignkeyable_macro;
mod query_operations;
mod utils;

use proc_macro::TokenStream as CompilerTokenStream;
use quote::quote;
use syn::DeriveInput;
use utils::{function_parser::FunctionParser, helpers, macro_tokens::MacroTokens};

use crate::canyon_entity_macro::generate_canyon_entity_tokens;
use crate::canyon_mapper_macro::canyon_mapper_impl_tokens;
use crate::foreignkeyable_macro::foreignkeyable_impl_tokens;
use crate::query_operations::impl_crud_operations_trait_for_struct;
use canyon_entities::{
    entity::CanyonEntity,
    manager_builder::{generate_enum_with_fields, generate_enum_with_fields_values},
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

/// Takes data from the struct annotated with the `canyon_entity` macro to fill the Canyon Register
/// where lives the data that Canyon needs to work.
///
/// Also, it's the responsible for generate the tokens for all the `Crud` methods available over
/// your type
#[proc_macro_attribute]
pub fn canyon_entity(meta: CompilerTokenStream, input: CompilerTokenStream) -> CompilerTokenStream {
    let attrs = syn::parse_macro_input!(meta as syn::AttributeArgs);
    generate_canyon_entity_tokens(attrs, input).into()
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

/// proc-macro for annotate struct fields that holds a foreign key relation.
///
/// So basically, if you have some `ForeignKey` attribute, annotate the parent
/// struct (where the ForeignKey table property points) with this macro
/// to make it able to work with compound table relations
#[proc_macro_derive(ForeignKeyable)]
pub fn implement_foreignkeyable_for_type(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    foreignkeyable_impl_tokens(ast).into()
}

#[proc_macro_derive(CanyonMapper)]
pub fn implement_row_mapper_for_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    canyon_mapper_impl_tokens(ast).into()
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
