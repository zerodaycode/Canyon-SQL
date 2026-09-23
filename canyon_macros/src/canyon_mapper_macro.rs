#![allow(unused_imports)]

use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{DeriveInput, GenericArgument, PathArguments, Type, Visibility};

use crate::utils::macro_tokens::MacroTokens;

use crate::MacroResult;

pub fn canyon_mapper_tokens(input: CompilerTokenStream) -> MacroResult {
    let ast = syn::parse::<DeriveInput>(input)?;
    let macro_data = MacroTokens::new(&ast)?;

    canyon_mapper_impl_tokens(macro_data)
}

/// Generates the [`canyon_sql::core::RowMapper`] and
/// [`canyon_sql::query::bounds::EntityRuntimeInfo`] implementations for an
/// entity annotated with `CanyonMapper`.
fn canyon_mapper_impl_tokens(ast: MacroTokens) -> MacroResult {
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
            ) -> canyon_sql::core::CanyonResult<Self::Output> {
                Ok(Self {
                    #(#field_mappings),*
                })
            }
        });
    }

    #[cfg(feature = "mssql")]
    {
        let field_mappings = create_sqlserver_fields_mapping(&ty_str, &fields)?;

        mapper_methods.extend(quote! {
            fn deserialize_sqlserver(
                row: &canyon_sql::db_clients::tiberius::Row,
            ) -> canyon_sql::core::CanyonResult<Self::Output> {
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
            ) -> canyon_sql::core::CanyonResult<Self::Output> {
                Ok(Self {
                    #(#field_mappings),*
                })
            }
        });
    }

    let entity_runtime_info = __details::entity_runtime_info_macro::tokens(&ast);

    Ok(quote! {
        use crate::canyon_sql::crud::CrudOperations;

        impl #impl_generics canyon_sql::core::RowMapper
            for #ty #ty_generics
            #where_clause
        {
            type Output = #ty;

            #mapper_methods
        }

        #entity_runtime_info
    })
}

#[cfg(feature = "postgres")]
fn create_postgres_fields_mapping<'a>(
    entity_name: &'a str,
    fields: &'a [(Visibility, Ident, Type)],
) -> impl Iterator<Item = TokenStream> + use<'a> {
    fields.iter().map(move |(_, ident, field_type)| {
        let column_name = ident.to_string();

        quote! {
            #ident: row
                .try_get::<&str, #field_type>(#column_name)
                .map_err(|source| canyon_sql::core::MappingError::postgres(
                    #entity_name,
                    #column_name,
                    source,
                ))?
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

        quote! {
            #ident: row
                .get_opt(#column_name)
                .ok_or_else(|| canyon_sql::core::MappingError::column_not_found(
                    #entity_name,
                    #column_name,
                    canyon_sql::connection::DatabaseType::MySQL,
                ))?
                .map_err(|source| canyon_sql::core::MappingError::mysql(
                    #entity_name,
                    #column_name,
                    source,
                ))?
        }
    })
}

#[cfg(feature = "mssql")]
fn create_sqlserver_fields_mapping<'a>(
    entity_name: &'a str,
    fields: &'a [(Visibility, Ident, Type)],
) -> syn::Result<Vec<TokenStream>> {
    fields
        .iter()
        .map(move |(_, ident, field_type)| {
            let column_name = ident.to_string();

            let deserialization =
                create_tiberius_field_deserialization(entity_name, field_type, &column_name)?;

            Ok(quote! {
                #ident: #deserialization
            })
        })
        .collect()
}

/// Builds the conversion required by Tiberius for fields whose borrowed SQL
/// representation differs from the entity's owned Rust type.
///
/// In particular, `String` fields are read as `&str` and then converted into
/// owned values.
#[cfg(feature = "mssql")]
fn create_tiberius_field_deserialization(
    entity_name: &str,
    target_type: &Type,
    column_name: &str,
) -> syn::Result<TokenStream> {
    let (deserializing_type, is_optional, convert_to_owned) =
        sqlserver_deserialization_type(target_type)?;

    let require_value = if is_optional {
        quote! {}
    } else {
        quote! {
            .ok_or_else(|| canyon_sql::core::MappingError::unexpected_null(
                #entity_name,
                #column_name,
                canyon_sql::connection::DatabaseType::SqlServer,
            ))?
        }
    };

    let convert_to_owned = if convert_to_owned {
        if is_optional {
            quote! { .map(ToOwned::to_owned) }
        } else {
            quote! { .to_owned() }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        row.try_get::<#deserializing_type, &str>(#column_name)
            .map_err(|source| canyon_sql::core::MappingError::sql_server(
                #entity_name,
                #column_name,
                source,
            ))?
            #require_value
            #convert_to_owned
    })
}

#[cfg(feature = "mssql")]
fn sqlserver_deserialization_type(target_type: &Type) -> syn::Result<(TokenStream, bool, bool)> {
    let optional_inner = option_inner_type(target_type)?;
    let (inner_type, is_optional) = optional_inner
        .map(|inner| (inner, true))
        .unwrap_or((target_type, false));

    let convert_to_owned = type_path_ends_with(inner_type, "String");
    let deserializing_type = if convert_to_owned {
        quote! { &str }
    } else {
        quote! { #inner_type }
    };

    Ok((deserializing_type, is_optional, convert_to_owned))
}

#[cfg(feature = "mssql")]
fn option_inner_type(target_type: &Type) -> syn::Result<Option<&Type>> {
    let Type::Path(type_path) = target_type else {
        return Ok(None);
    };
    let Some(segment) = type_path.path.segments.last() else {
        return Ok(None);
    };

    if segment.ident != "Option" {
        return Ok(None);
    }

    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Option fields must declare exactly one inner type",
        ));
    };

    let mut type_arguments = arguments.args.iter().filter_map(|argument| match argument {
        GenericArgument::Type(inner_type) => Some(inner_type),
        _ => None,
    });
    let inner_type = type_arguments.next();

    if arguments.args.len() != 1 || inner_type.is_none() || type_arguments.next().is_some() {
        return Err(syn::Error::new_spanned(
            target_type,
            "Option fields must declare exactly one inner type",
        ));
    }

    Ok(inner_type)
}

#[cfg(feature = "mssql")]
fn type_path_ends_with(target_type: &Type, expected: &str) -> bool {
    matches!(
        target_type,
        Type::Path(type_path)
            if type_path
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == expected)
    )
}

#[cfg(all(test, feature = "mssql"))]
mod mapper_macro_tests {
    use super::sqlserver_deserialization_type;
    use syn::Type;

    fn parse_type(source: &str) -> Type {
        syn::parse_str(source).unwrap()
    }

    #[test]
    fn maps_owned_strings_to_tiberius_borrowed_strings() {
        let (ty, optional, owned) = sqlserver_deserialization_type(&parse_type("String")).unwrap();
        assert_eq!("& str", ty.to_string());
        assert!(!optional);
        assert!(owned);

        let (ty, optional, owned) =
            sqlserver_deserialization_type(&parse_type("std::option::Option<std::string::String>"))
                .unwrap();
        assert_eq!("& str", ty.to_string());
        assert!(optional);
        assert!(owned);
    }

    #[test]
    fn preserves_qualified_and_generic_field_types() {
        let (ty, optional, owned) =
            sqlserver_deserialization_type(&parse_type("Option<chrono::DateTime<chrono::Utc>>"))
                .unwrap();
        assert_eq!("chrono :: DateTime < chrono :: Utc >", ty.to_string());
        assert!(optional);
        assert!(!owned);

        let (ty, optional, owned) =
            sqlserver_deserialization_type(&parse_type("domain::UserId")).unwrap();
        assert_eq!("domain :: UserId", ty.to_string());
        assert!(!optional);
        assert!(!owned);
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
                    ) -> canyon_sql::CanyonResult<()> {
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
                    Err(canyon_sql::error::QueryBuilderError::MissingPrimaryKey.into())
                }
            }
        }
    }
}
