extern crate proc_macro;

mod canyon_entity_macro;
mod canyon_macro;
mod query_operations;
mod utils;

use canyon_connection::CANYON_TOKIO_RUNTIME;
use canyon_entity_macro::parse_canyon_entity_proc_macro_attr;
use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{DeriveInput, Fields, Type, Visibility};

use query_operations::{
    delete::{generate_delete_query_tokens, generate_delete_tokens},
    insert::{generate_insert_tokens, generate_multiple_insert_tokens},
    select::{
        generate_count_tokens, generate_find_all_query_tokens, generate_find_all_tokens,
        generate_find_all_unchecked_tokens, generate_find_by_foreign_key_tokens,
        generate_find_by_pk_tokens, generate_find_by_reverse_foreign_key_tokens,
    },
    update::{generate_update_query_tokens, generate_update_tokens},
};

use canyon_macro::{parse_canyon_macro_attributes, wire_queries_to_execute};
use utils::{function_parser::FunctionParser, helpers, macro_tokens::MacroTokens};

use canyon_migrations::{
    manager::{
        entity::CanyonEntity,
        manager_builder::{
            generate_enum_with_fields, generate_enum_with_fields_values, generate_user_struct,
        },
    },
    migrations::handler::Migrations,
};

use canyon_migrations::{
    migrations::register_types::{CanyonRegisterEntity, CanyonRegisterEntityField},
    CANYON_REGISTER_ENTITIES,
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
    let attrs = syn::parse_macro_input!(_meta as syn::AttributeArgs);

    // Parses the attributes declared in the arguments of this proc macro
    let attrs_parse_result = parse_canyon_macro_attributes(&attrs);
    if attrs_parse_result.error.is_some() {
        return attrs_parse_result.error.unwrap();
    }

    // Parses the function items that this attribute is attached to
    let func_res = syn::parse::<FunctionParser>(input);
    if func_res.is_err() {
        return quote! { fn main() {} }.into();
    }

    // TODO check if the `canyon` macro it's attached only to main?
    let func = func_res.ok().unwrap();
    let sign = func.sig;
    let body = func.block.stmts;

    if attrs_parse_result.allowed_migrations {
        CANYON_TOKIO_RUNTIME.block_on(async {
            canyon_connection::init_connections_cache().await;
            Migrations::migrate().await;
        });

        // The queries to execute at runtime in the managed state
        let mut queries_tokens: Vec<TokenStream> = Vec::new();
        wire_queries_to_execute(&mut queries_tokens);

        // The final code wired in main()
        quote! {
            #sign {
                canyon_sql::runtime::CANYON_TOKIO_RUNTIME
                    .handle()
                    .block_on( async {
                        canyon_sql::runtime::init_connections_cache().await;
                        {
                            #(#queries_tokens)*
                        }
                        #(#body)*
                    }
                )
            }
        }
        .into()
    } else {
        quote! {
            #sign {
                canyon_sql::runtime::CANYON_TOKIO_RUNTIME
                .handle()
                .block_on( async {
                        canyon_sql::runtime::init_connections_cache().await;
                        #(#body)*
                    }
                )
            }
        }
        .into()
    }
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
        use canyon_sql::crud::bounds::QueryParameter;
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
    let entity = entity_res.expect("Unexpected error parsing the struct");
    // Generate the bits of code that we should give back to the compiler
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
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate

    // Calls the helper struct to build the tokens that generates the final CRUD methods
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
    let ty = macro_data.ty;

    // Builds the find_all() query
    let _find_all_unchecked_tokens =
        generate_find_all_unchecked_tokens(macro_data, &table_schema_data);
    // Builds the find_all_result() query
    let _find_all_tokens = generate_find_all_tokens(macro_data, &table_schema_data);
    // Builds the find_all_query() query as a QueryBuilder
    let _find_all_query_tokens = generate_find_all_query_tokens(macro_data, &table_schema_data);

    // Builds a COUNT(*) query over some table
    let _count_tokens = generate_count_tokens(macro_data, &table_schema_data);

    // Builds the find_by_pk() query
    let _find_by_pk_tokens = generate_find_by_pk_tokens(macro_data, &table_schema_data);

    // Builds the insert() query
    let _insert_tokens = generate_insert_tokens(macro_data, &table_schema_data);
    // Builds the insert_multi() query
    let _insert_multi_tokens = generate_multiple_insert_tokens(macro_data, &table_schema_data);

    // Builds the update() queries
    let _update_tokens = generate_update_tokens(macro_data, &table_schema_data);
    // Builds the update() query as a QueryBuilder
    let _update_query_tokens = generate_update_query_tokens(macro_data, &table_schema_data);

    // Builds the delete() queries
    let _delete_tokens = generate_delete_tokens(macro_data, &table_schema_data);

    // Builds the delete() query as a QueryBuilder
    let _delete_query_tokens = generate_delete_query_tokens(macro_data, &table_schema_data);

    // Search by foreign (d) key as Vec, cause Canyon supports multiple fields having FK annotation
    let _search_by_fk_tokens: Vec<(TokenStream, TokenStream)> =
        generate_find_by_foreign_key_tokens(macro_data);
    let fk_method_signatures = _search_by_fk_tokens.iter().map(|(sign, _)| sign);
    let fk_method_implementations = _search_by_fk_tokens.iter().map(|(_, m_impl)| m_impl);

    // The tokens for generating the methods that enable Canyon to retrieve the child entities that are of T type
    // given a parent entity U: ForeignKeyable, as an associated function for the child type (T)
    let _search_by_revese_fk_tokens: Vec<(TokenStream, TokenStream)> =
        generate_find_by_reverse_foreign_key_tokens(macro_data, &table_schema_data);
    let rev_fk_method_signatures = _search_by_revese_fk_tokens.iter().map(|(sign, _)| sign);
    let rev_fk_method_implementations =
        _search_by_revese_fk_tokens.iter().map(|(_, m_impl)| m_impl);

    // The autogenerated name for the trait that holds the fk and rev fk searches
    let fk_trait_ident = Ident::new(
        &format!("{}FkOperations", &ty.to_string()),
        proc_macro2::Span::call_site(),
    );

    let crud_operations_tokens = quote! {
        // The find_all_result impl
        #_find_all_tokens

        // The find_all impl
        #_find_all_unchecked_tokens

        // The find_all_query impl
        #_find_all_query_tokens

        // The COUNT(*) impl
        #_count_tokens

        // The find_by_pk impl
        #_find_by_pk_tokens

        // The insert impl
        #_insert_tokens

        // The insert of multiple entities impl
        #_insert_multi_tokens

        // The update impl
        #_update_tokens

        // The update as a querybuilder impl
        #_update_query_tokens

        // The delete impl
        #_delete_tokens

        // The delete as querybuilder impl
        #_delete_query_tokens
    };

    let tokens = if !_search_by_fk_tokens.is_empty() {
        quote! {
            #[canyon_sql::macros::async_trait]
            impl canyon_sql::crud::CrudOperations<#ty> for #ty {
                #crud_operations_tokens
            }

            impl canyon_sql::crud::Transaction<#ty> for #ty {}

            /// Hidden trait for generate the foreign key operations available
            /// in Canyon without have to define them before hand in CrudOperations
            /// because it's just impossible with the actual system (where the methods
            /// are generated dynamically based on some properties of the `foreign_key`
            /// annotation)
            #[canyon_sql::macros::async_trait]
            pub trait #fk_trait_ident<#ty> {
                #(#fk_method_signatures)*
                #(#rev_fk_method_signatures)*
            }
            #[canyon_sql::macros::async_trait]
            impl #fk_trait_ident<#ty> for #ty
                where #ty:
                    std::fmt::Debug +
                    canyon_sql::crud::CrudOperations<#ty> +
                    canyon_sql::crud::RowMapper<#ty>
            {
                #(#fk_method_implementations)*
                #(#rev_fk_method_implementations)*
            }
        }
    } else {
        quote! {
            #[canyon_sql::macros::async_trait]
            impl canyon_sql::crud::CrudOperations<#ty> for #ty {
                #crud_operations_tokens
            }

            impl canyon_sql::crud::Transaction<#ty> for #ty {}
        }
    };

    tokens.into()
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
            #i => Some(&self.#ident as &dyn canyon_sql::crud::bounds::QueryParameter<'_>)
        }
    });
    let field_idents_cloned = field_idents.clone();

    quote! {
        /// Implementation of the trait `ForeignKeyable` for the type
        /// calling this derive proc macro
        impl canyon_sql::crud::bounds::ForeignKeyable<Self> for #ty {
            fn get_fk_column(&self, column: &str) -> Option<&dyn canyon_sql::crud::bounds::QueryParameter<'_>> {
                match column {
                    #(#field_idents),*,
                    _ => None
                }
            }
        }
        /// Implementation of the trait `ForeignKeyable` for a reference of this type
        /// calling this derive proc macro
        impl canyon_sql::crud::bounds::ForeignKeyable<&Self> for &#ty {
            fn get_fk_column<'a>(&self, column: &'a str) -> Option<&dyn canyon_sql::crud::bounds::QueryParameter<'_>> {
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
    // Gets the data from the AST
    let ast: DeriveInput = syn::parse(input).unwrap();

    // Recovers the identifiers of the structs members
    let fields = fields_with_types(match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => {
            return syn::Error::new(ast.ident.span(), "CanyonMapper only works with Structs")
                .to_compile_error()
                .into()
        }
    });

    // Here it's where the incoming values of the DatabaseResult are wired into a new
    // instance, mapping the fields of the type against the columns
    let init_field_values = fields.iter().map(|(_vis, ident, _ty)| {
        let ident_name = ident.to_string();
        quote! {
            #ident: row.try_get(#ident_name)
                .expect(format!("Failed to retrieve the {} field", #ident_name).as_ref())
        }
    });

    let init_field_values_sqlserver = fields.iter().map(|(_vis, ident, ty)| {
        let ident_name = ident.to_string();

        if get_field_type_as_string(ty) == "String" {
            quote! {
                #ident: row.get::<&str, &str>(#ident_name)
                    .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref())
                    .to_string()
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<i64>" {
            quote! {
                #ident: row.get::<i64, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<i32>" {
            quote! {
                #ident: row.get::<i32, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<i16>" {
            quote! {
                #ident: row.get::<i16, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<f32>" {
            quote! {
                #ident: row.get::<f32, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<f64>" {
            quote! {
                #ident: row.get::<f64, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<String>" {
            quote! {
                #ident: row.get::<&str, &str>(#ident_name)
                    .map( |x| x.to_owned() )
            }
        } else if get_field_type_as_string(ty) == "NaiveDate" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::NaiveDate, &str>(#ident_name)
                    .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref())
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<NaiveDate>" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::NaiveDate, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty) == "NaiveTime" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::NaiveTime, &str>(#ident_name)
                    .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref())
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<NaiveTime>" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::NaiveTime, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty) == "NaiveDateTime" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::NaiveDateTime, &str>(#ident_name)
                    .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref())
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<NaiveDateTime>" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::NaiveDateTime, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty) == "DateTime" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::DateTime, &str>(#ident_name)
                    .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref())
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<DateTime>" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::DateTime, &str>(#ident_name)
            }
        } else {
            quote! {
                #ident: row.get::<#ty, &str>(#ident_name)
                    .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref())
            }
        }
    });

    // The type of the Struct
    let ty = ast.ident;

    let postgres_enabled = cfg!(feature = "postgres");
    let mssql_enabled = cfg!(feature = "mssql");

    let tokens = if postgres_enabled && mssql_enabled {
        quote! {
            impl canyon_sql::crud::RowMapper<Self> for #ty {
                fn deserialize_postgresql(row: &canyon_sql::db_clients::tokio_postgres::Row) -> #ty {
                    Self {
                        #(#init_field_values),*
                    }
                }
                fn deserialize_sqlserver(row: &canyon_sql::db_clients::tiberius::Row) -> #ty {
                    Self {
                        #(#init_field_values_sqlserver),*
                    }
                }
            }
        }
    } else if postgres_enabled {
        quote! {
            impl canyon_sql::crud::RowMapper<Self> for #ty {
                fn deserialize_postgresql(row: &canyon_sql::db_clients::tokio_postgres::Row) -> #ty {
                    Self {
                        #(#init_field_values),*
                    }
                }
            }
        }
    } else if mssql_enabled {
        quote! {
            impl canyon_sql::crud::RowMapper<Self> for #ty {
                fn deserialize_sqlserver(row: &canyon_sql::db_clients::tiberius::Row) -> #ty {
                    Self {
                        #(#init_field_values_sqlserver),*
                    }
                }
            }
        }
    } else {
        quote! {
            panic!(
                "Reached a branch in the implementation of the Row Mapper macro that should never be reached.\
                This is a severe bug of Canyon-SQL. Please, open us an issue at \
                https://github.com/zerodaycode/Canyon-SQL/issues and let us know about that failure."
            )
        }
    };

    tokens.into()
}

/// Helper for generate the fields data for the Custom Derives Macros
fn filter_fields(fields: &Fields) -> Vec<(Visibility, Ident)> {
    fields
        .iter()
        .map(|field| (field.vis.clone(), field.ident.as_ref().unwrap().clone()))
        .collect::<Vec<_>>()
}

fn fields_with_types(fields: &Fields) -> Vec<(Visibility, Ident, Type)> {
    fields
        .iter()
        .map(|field| {
            (
                field.vis.clone(),
                field.ident.as_ref().unwrap().clone(),
                field.ty.clone(),
            )
        })
        .collect::<Vec<_>>()
}

fn get_field_type_as_string(typ: &Type) -> String {
    match typ {
        Type::Array(type_) => type_.to_token_stream().to_string(),
        Type::BareFn(type_) => type_.to_token_stream().to_string(),
        Type::Group(type_) => type_.to_token_stream().to_string(),
        Type::ImplTrait(type_) => type_.to_token_stream().to_string(),
        Type::Infer(type_) => type_.to_token_stream().to_string(),
        Type::Macro(type_) => type_.to_token_stream().to_string(),
        Type::Never(type_) => type_.to_token_stream().to_string(),
        Type::Paren(type_) => type_.to_token_stream().to_string(),
        Type::Path(type_) => type_.to_token_stream().to_string(),
        Type::Ptr(type_) => type_.to_token_stream().to_string(),
        Type::Reference(type_) => type_.to_token_stream().to_string(),
        Type::Slice(type_) => type_.to_token_stream().to_string(),
        Type::TraitObject(type_) => type_.to_token_stream().to_string(),
        Type::Tuple(type_) => type_.to_token_stream().to_string(),
        Type::Verbatim(type_) => type_.to_token_stream().to_string(),
        _ => "".to_owned(),
    }
}
