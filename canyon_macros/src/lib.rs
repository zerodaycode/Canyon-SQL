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
mod canyon_tokio_test;

use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, parse_macro_input};

use canyon_entities::{
    entity::CanyonEntity,
    manager_builder::{
        generate_enum_with_fields,
        generate_enum_with_fields_values,
        generated_enum_type_for_struct_data,
    },
};
use crate::{
    canyon_tokio_test::generate_canyon_tokio_test_tokens,
    canyon_mapper_macro::canyon_mapper_tokens,
    canyon_entity_macro::{
        CanyonEntityAttributeArgs,
        generate_canyon_entity_tokens,
    },
    query_operations::{
        impl_crud_operations_trait_for_struct,
        impl_crud_entity_operations_trait_for_struct,
        read::generate_read_operations_tokens,
        update::generate_update_method_tokens,
        delete::generate_delete_method_tokens
    },
    utils::{
        function_parser::FunctionParser,
        helpers,
        macro_tokens::MacroTokens,
    },
    foreignkeyable_macro::foreignkeyable_tokens,
    query_operations::insert::generate_insert_method_tokens
};

type MacroResult = syn::Result<TokenStream>;

type OperationsGenerator =
    for<'a> fn(&MacroTokens<'a>, &str) -> MacroResult;

/// Parses the derive input and delegates token generation to one of the
/// operation-specific generators.
///
/// Errors are converted into `compile_error!` only at the proc-macro boundary.
/// The internal generators can therefore propagate `syn::Error` with `?`.
fn derive_operations(
    input: CompilerTokenStream,
    generator: OperationsGenerator,
) -> CompilerTokenStream {
    derive_operations_tokens(input, generator)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn derive_operations_tokens(
    input: CompilerTokenStream,
    generator: OperationsGenerator,
) -> MacroResult {
    let ast = syn::parse::<DeriveInput>(input)?;
    let macro_data = MacroTokens::new(&ast)?;

    let table_schema_data = helpers::table_schema_parser(&macro_data)
        .map_err(|tokens| {
            Error::new_spanned(
                tokens,
                "failed to parse Canyon table and schema metadata",
            )
        })?;

    generator(&macro_data, &table_schema_data.sql())
}

/// Canyon's application entry point.
///
/// Initializes Canyon inside its Tokio runtime before executing the user's
/// `main` body and, when enabled, runs the generated migration setup.
#[proc_macro_attribute]
pub fn main(
    _meta: CompilerTokenStream,
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    let function = parse_macro_input!(input as FunctionParser);

    if function.sig.ident != "main" {
        return Error::new(
            function.sig.ident.span(),
            "the #[canyon::main] attribute can only be applied to `fn main()`",
        )
            .into_compile_error()
            .into();
    }

    let signature = function.sig;
    let visibility = function.vis;
    let attributes = function.attrs;
    let body = function.block.stmts;

    #[allow(unused_mut, unused_assignments)]
    let mut migrations_tokens = quote! {};

    #[cfg(feature = "migrations")]
    {
        migrations_tokens = main_with_queries();
    }

    quote! {
        #(#attributes)*
        #visibility #signature {
            canyon_sql::runtime::get_canyon_tokio_runtime()
                .handle()
                .block_on(async {
                    canyon_sql::core::Canyon::init()
                        .await
                        .expect("error initializing Canyon's connection pools");

                    #migrations_tokens
                    #(#body)*
                })
        }
    }
        .into()
}

/// Runs a test function inside Canyon's Tokio runtime.
#[proc_macro_attribute]
pub fn canyon_tokio_test(
    _meta: CompilerTokenStream,
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    generate_canyon_tokio_test_tokens(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Registers the table metadata and runtime field information required by
/// Canyon.
#[proc_macro_attribute]
pub fn canyon_entity(
    meta: CompilerTokenStream,
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    let attributes = parse_macro_input!(
        meta with CanyonEntityAttributeArgs::parse_terminated
    );

    generate_canyon_entity_tokens(attributes, input).into()
}

/// Derives Canyon's complete CRUD API.
///
/// This is the convenience derive. It includes both:
///
/// - operations tied directly to the annotated type;
/// - operations that accept a separate Canyon entity, as used by repository
///   adapters configured through `#[canyon_crud(maps_to = Entity)]`.
#[proc_macro_derive(CanyonCrud, attributes(canyon_crud))]
pub fn canyon_crud(
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    derive_operations(
        input,
        impl_crud_operations_trait_for_struct,
    )
}

/// Derives read operations tied directly to the annotated type.
///
/// This includes operations such as `find_all`, `find_by_pk`, `count` and
/// `select_query`.
#[proc_macro_derive(CanyonRead, attributes(canyon_crud))]
pub fn canyon_read(
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    // TODO: we should split the find_by_pk with the entity methods
    derive_operations(
        input,
        generate_read_operations_tokens,
    )
}

/// Derives insertion of instances of the annotated type.
#[proc_macro_derive(CanyonInsert, attributes(canyon_crud))]
pub fn canyon_insert(
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    derive_operations(
        input,
        generate_insert_method_tokens,
    )
}

/// Derives update operations tied directly to instances of the annotated type.
#[proc_macro_derive(CanyonUpdate, attributes(canyon_crud))]
pub fn canyon_update(
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    derive_operations(
        input,
        generate_update_method_tokens,
    )
}

/// Derives delete operations tied directly to instances of the annotated type.
#[proc_macro_derive(CanyonDelete, attributes(canyon_crud))]
pub fn canyon_delete(
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    derive_operations(
        input,
        generate_delete_method_tokens,
    )
}

/// Derives every operation that works with a separate Canyon entity.
///
/// This API is intended for repository adapters and layered architectures where
/// the type performing persistence is not itself the domain entity being
/// persisted.
#[proc_macro_derive(CanyonEntityCrud, attributes(canyon_crud))]
pub fn canyon_entity_crud(
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    derive_operations(
        input,
        impl_crud_entity_operations_trait_for_struct,
    )
}

/// Derives the metadata required to navigate foreign-key relationships.
#[proc_macro_derive(ForeignKeyable)]
pub fn implement_foreignkeyable_for_type(
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    foreignkeyable_tokens(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derives database-row deserialization for the annotated type.
#[proc_macro_derive(CanyonMapper)]
pub fn implement_row_mapper_for_type(
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    canyon_mapper_tokens(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Generates the field identifiers used by Canyon's typed query builder.
#[proc_macro_derive(Fields)]
pub fn querybuilder_fields(
    input: CompilerTokenStream,
) -> CompilerTokenStream {
    querybuilder_fields_tokens(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn querybuilder_fields_tokens(
    input: CompilerTokenStream,
) -> MacroResult {
    let entity = syn::parse::<CanyonEntity>(input)?;

    let struct_metadata =
        generated_enum_type_for_struct_data(&entity);

    let fields =
        generate_enum_with_fields(&entity);

    let field_values =
        generate_enum_with_fields_values(&entity);

    Ok(quote! {
        use canyon_sql::query::bounds::EntityTable;
        use canyon_sql::query::bounds::FieldIdentifier;

        #struct_metadata
        #fields
        #field_values
    })
}
