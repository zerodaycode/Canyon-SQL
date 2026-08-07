#![allow(unused_imports)]

use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::{DeriveInput, Type, Visibility};

use crate::utils::macro_tokens::MacroTokens;
use canyon_core::connection::database_type::DatabaseType;

#[cfg(feature = "mssql")]
use quote::ToTokens;

use crate::MacroResult;

#[cfg(feature = "mssql")]
const BY_VALUE_CONVERSION_TARGETS: [&str; 1] = ["String"];

pub fn canyon_mapper_tokens(input: CompilerTokenStream) -> MacroResult {
    let ast = syn::parse::<DeriveInput>(input)?;
    let macro_data = MacroTokens::new(&ast)?;

    Ok(canyon_mapper_impl_tokens(macro_data))
}

/// Generates the [`canyon_sql::core::RowMapper`] and
/// [`canyon_sql::query::bounds::EntityRuntimeInfo`] implementations for an
/// entity annotated with `CanyonMapper`.
fn canyon_mapper_impl_tokens(ast: MacroTokens) -> TokenStream {
    let ty = ast.ty;
    let ty_str = ty.to_string();
    let fields = ast.fields();
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut mapper_methods = TokenStream::new();

    #[cfg(feature = "postgres")]
    {
        let field_mappings = create_postgres_fields_mapping(&ty_str, &fields);

        mapper_methods.extend(quote! {
            fn deserialize_postgresql(
                row: &canyon_sql::db_clients::tokio_postgres::Row,
            ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Self {
                    #(#field_mappings),*
                })
            }
        });
    }

    #[cfg(feature = "mssql")]
    {
        let field_mappings = create_sqlserver_fields_mapping(&ty_str, &fields);

        mapper_methods.extend(quote! {
            fn deserialize_sqlserver(
                row: &canyon_sql::db_clients::tiberius::Row,
            ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Self {
                    #(#field_mappings),*
                })
            }
        });
    }

    #[cfg(feature = "mysql")]
    {
        let field_mappings = create_mysql_fields_mapping(&ty_str, &fields);

        mapper_methods.extend(quote! {
            fn deserialize_mysql(
                row: &canyon_sql::db_clients::mysql_async::Row,
            ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Self {
                    #(#field_mappings),*
                })
            }
        });
    }

    let entity_runtime_info = __details::entity_runtime_info_macro::tokens(&ast);

    quote! {
        use crate::canyon_sql::crud::CrudOperations;

        impl #impl_generics canyon_sql::core::RowMapper
            for #ty #ty_generics
            #where_clause
        {
            type Output = #ty;

            #mapper_methods
        }

        #entity_runtime_info
    }
}

#[cfg(feature = "postgres")]
fn create_postgres_fields_mapping<'a>(
    entity_name: &'a str,
    fields: &'a [(Visibility, Ident, Type)],
) -> impl Iterator<Item = TokenStream> + use<'a> {
    fields.iter().map(move |(_, ident, field_type)| {
        let column_name = ident.to_string();
        let error =
            create_row_mapper_error_extracting_row(ident, entity_name, DatabaseType::PostgreSql);

        quote! {
            #ident: row
                .try_get::<&str, #field_type>(#column_name)
                .map_err(|_| #error)?
        }
    })
}

#[cfg(feature = "mysql")]
fn create_mysql_fields_mapping<'a>(
    entity_name: &'a str,
    fields: &'a [(Visibility, Ident, Type)],
) -> impl Iterator<Item = TokenStream> + use<'a> {
    fields.iter().map(move |(_, ident, _)| {
        let column_name = ident.to_string();
        let error = create_row_mapper_error_extracting_row(ident, entity_name, DatabaseType::MySQL);

        quote! {
            #ident: row
                .get_opt(#column_name)
                .ok_or_else(|| #error)??
        }
    })
}

#[cfg(feature = "mssql")]
fn create_sqlserver_fields_mapping<'a>(
    entity_name: &'a str,
    fields: &'a [(Visibility, Ident, Type)],
) -> impl Iterator<Item = TokenStream> + use<'a> {
    fields.iter().map(move |(_, ident, field_type)| {
        let column_name = ident.to_string();
        let error =
            create_row_mapper_error_extracting_row(ident, entity_name, DatabaseType::SqlServer);

        let target_type = get_field_type_as_string(field_type);
        let deserialization =
            create_tiberius_field_deserialization(&target_type, &column_name, error);

        quote! {
            #ident: #deserialization
        }
    })
}

/// Builds the conversion required by Tiberius for fields whose borrowed SQL
/// representation differs from the entity's owned Rust type.
///
/// In particular, `String` fields are read as `&str` and then converted into
/// owned values.
#[cfg(feature = "mssql")]
fn create_tiberius_field_deserialization(
    target_type: &str,
    column_name: &str,
    error: String,
) -> TokenStream {
    let is_optional = target_type.contains("Option");

    let require_value = if is_optional {
        quote! {}
    } else {
        quote! { .ok_or_else(|| #error)? }
    };

    let deserializing_type = get_deserializing_type(target_type);

    let convert_to_owned = if BY_VALUE_CONVERSION_TARGETS
        .iter()
        .any(|candidate| target_type.contains(candidate))
    {
        if is_optional {
            quote! { .map(ToOwned::to_owned) }
        } else {
            quote! { .to_owned() }
        }
    } else {
        quote! {}
    };

    quote! {
        row.get::<#deserializing_type, &str>(#column_name)
            #require_value
            #convert_to_owned
    }
}

#[cfg(feature = "mssql")]
fn extract_deserializing_type_name(target_type: &str) -> String {
    static TYPE_REGEX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();

    let regex = TYPE_REGEX.get_or_init(|| {
        Regex::new(r"(?:Option\s*<\s*)?(?P<type>&?\w+)(?:\s*>)?")
            .expect("the Tiberius type extraction regex must be valid")
    });

    regex
        .captures(target_type)
        .map(|captures| captures["type"].to_owned())
        .unwrap_or_else(|| {
            panic!("Unable to determine the SQL Server deserialization type for `{target_type}`")
        })
}

#[cfg(feature = "mssql")]
fn get_deserializing_type(target_type: &str) -> TokenStream {
    let extracted_type = extract_deserializing_type_name(target_type);

    if BY_VALUE_CONVERSION_TARGETS.contains(&extracted_type.as_str()) {
        quote! { &str }
    } else if extracted_type.contains("Date") || extracted_type.contains("Time") {
        let ident = Ident::new(&extracted_type, Span::call_site());
        quote! { canyon_sql::date_time::#ident }
    } else {
        let ident = Ident::new(&extracted_type, Span::call_site());
        quote! { #ident }
    }
}

#[cfg(feature = "mssql")]
fn get_field_type_as_string(field_type: &Type) -> String {
    field_type.to_token_stream().to_string()
}

fn create_row_mapper_error_extracting_row(
    field_ident: &Ident,
    entity_name: &str,
    database_type: DatabaseType,
) -> String {
    std::io::Error::other(format!(
        "Failed to retrieve field `{field_ident}` for entity `{entity_name}` using {database_type}"
    ))
    .to_string()
}

#[cfg(all(test, feature = "mssql"))]
mod mapper_macro_tests {
    use super::{extract_deserializing_type_name, get_deserializing_type};

    #[test]
    fn extracts_the_inner_tiberius_deserialization_type_name() {
        assert_eq!("String", extract_deserializing_type_name("String"));
        assert_eq!("String", extract_deserializing_type_name("Option<String>"));
        assert_eq!("i64", extract_deserializing_type_name("i64"));
        assert_eq!("DateTime", extract_deserializing_type_name("DateTime"));
        assert_eq!(
            "NaiveDateTime",
            extract_deserializing_type_name("NaiveDateTime")
        );
    }

    #[test]
    fn maps_canyon_types_to_tiberius_deserialization_tokens() {
        assert_eq!("& str", get_deserializing_type("String").to_string());
        assert_eq!(
            "& str",
            get_deserializing_type("Option<String>").to_string()
        );
        assert_eq!("i64", get_deserializing_type("i64").to_string());

        assert_eq!(
            "canyon_sql :: date_time :: DateTime",
            get_deserializing_type("DateTime").to_string()
        );
        assert_eq!(
            "canyon_sql :: date_time :: NaiveDateTime",
            get_deserializing_type("NaiveDateTime").to_string()
        );
    }
}

mod __details {
    use super::*;

    pub(crate) mod entity_runtime_info_macro {
        use super::*;
        use crate::utils::helpers;

        /// Generates runtime field access for CRUD operations.
        ///
        /// Insertable fields deliberately exclude the primary key. Primary-key
        /// metadata and access are exposed separately so callers can handle
        /// entities with and without generated keys.
        pub(crate) fn tokens(ast: &MacroTokens) -> TokenStream {
            let ty = ast.ty;
            let ty_str = ty.to_string();
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

            let primary_key = ast.get_primary_key_field_annotation();
            let primary_key_ident = primary_key.map(|field| field.ident);
            let primary_key_type = primary_key.map(|field| field.ty);

            let insertable_fields = ast.get_fields_idents_skipping_pk().collect::<Vec<_>>();
            let field_values = insertable_fields
                .iter()
                .map(|ident| quote! { &self.#ident });

            let field_columns = helpers::get_struct_fields_as_column_ref_token_stream(ast, true);

            let primary_key_name = primary_key_name_tokens(ast);
            let primary_key_value = primary_key_value_tokens(&primary_key_ident);
            let primary_key_type = primary_key_associated_type_tokens(&primary_key_type);
            let set_primary_key = set_primary_key_method_tokens(&primary_key_ident);

            quote! {
                impl #impl_generics canyon_sql::query::bounds::EntityRuntimeInfo for #ty #ty_generics #where_clause {
                    type PrimaryKey = #primary_key_type;

                    fn field_values(
                        &self,
                    ) -> Vec<&dyn canyon_sql::query::QueryParameter> {
                        vec![#(#field_values),*]
                    }

                    fn field_columns(
                    ) -> Vec<canyon_sql::query::ColumnRef<'static>> {
                        #field_columns.collect()
                    }

                    fn primary_key_name() -> Option<&'static str> {
                        #primary_key_name
                    }

                    fn primary_key_value(
                        &self,
                    ) -> Option<&dyn canyon_sql::query::QueryParameter> {
                        #primary_key_value
                    }

                    fn set_primary_key(
                        &mut self,
                        value: Self::PrimaryKey,
                    ) -> Result<
                        (),
                        Box<dyn std::error::Error + Send + Sync>,
                    > {
                        #set_primary_key
                    }

                    fn primary_key_column() -> Option<canyon_sql::query::ColumnRef<'static>> {
                        Self::primary_key_name()
                            .map(|pk| canyon_sql::query::ColumnRef::new(#ty_str, pk))
                    }
                }
            }
        }
    }

    fn primary_key_name_tokens(ast: &MacroTokens) -> TokenStream {
        match ast.get_primary_key_annotation() {
            Some(primary_key) => quote! { Some(#primary_key) },
            None => quote! { None },
        }
    }

    fn primary_key_value_tokens(primary_key_ident: &Option<&Ident>) -> TokenStream {
        match primary_key_ident {
            Some(ident) => {
                quote! {
                    Some(&self.#ident as &dyn canyon_sql::query::QueryParameter)
                }
            }
            None => quote! { None },
        }
    }

    fn primary_key_associated_type_tokens(primary_key_type: &Option<&Type>) -> TokenStream {
        match primary_key_type {
            Some(primary_key_type) => quote! { #primary_key_type },

            // The associated type remains mandatory even for entities without a
            // primary key. It is never consumed because `primary_key_value`
            // returns `None` and `set_primary_key` returns an error.
            None => quote! { i64 },
        }
    }

    fn set_primary_key_method_tokens(primary_key_ident: &Option<&Ident>) -> TokenStream {
        match primary_key_ident {
            Some(ident) => {
                quote! {
                    self.#ident = value.into();
                    Ok(())
                }
            }
            None => {
                quote! {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "No primary key field is defined for this entity",
                    )
                    .into())
                }
            }
        }
    }
}
