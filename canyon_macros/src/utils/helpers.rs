use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Fields, MetaNameValue, Token, Type, TypeGenerics, Visibility, punctuated::Punctuated,
};

use super::macro_tokens::MacroTokens;

/// Given the derived type of CrudOperations, and the possible mapping type if the `#[canyon_crud(maps_to=<Ident>]` exists,
/// returns a [`TokenStream`] with the final `RowMapper` implementor.
pub fn compute_crud_ops_mapping_target_type_with_generics(
    row_mapper_ty: &Ident,
    row_mapper_ty_generics: &TypeGenerics,
    crud_ops_ty: Option<&Ident>,
) -> TokenStream {
    if let Some(crud_ops_ty) = crud_ops_ty {
        quote! { #crud_ops_ty }
    } else {
        quote! { #row_mapper_ty #row_mapper_ty_generics }
    }
}

pub fn filter_fields(fields: &Fields) -> Vec<(Visibility, Ident)> {
    fields
        .iter()
        .map(|field| (field.vis.clone(), field.ident.as_ref().unwrap().clone()))
        .collect::<Vec<_>>()
}

pub fn fields_with_types(fields: &Fields) -> Vec<(Visibility, Ident, Type)> {
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

/// If the `canyon_entity` macro has valid attributes attached, and those attrs are the
/// user's desired `table_name` and/or the `schema_name`, this method returns its
/// correct form to be wired as the table name that the CRUD methods requires for generate
/// the queries
pub fn table_schema_parser(macro_data: &MacroTokens<'_>) -> Result<String, TokenStream> {
    let mut table_name: Option<String> = None;
    let mut schema: Option<String> = None;

    for attr in macro_data.attrs {
        let mut segments = attr.path.segments.iter();
        if segments.any(|seg| seg.ident == "canyon_macros" || seg.ident == "canyon_entity") {
            parse_canyon_entity_attr(attr, &mut schema, &mut table_name)?;
        }
        // TODO: if segments because we could parse here the canyon_crud proc_macro_attr
        // TODO: create a custom struct for hold this pair of data
    }

    let mut final_table_name = String::new();
    if schema.is_some() {
        final_table_name.push_str(format!("{}.", schema.unwrap()).as_str())
    }

    if let Some(t_name) = table_name {
        final_table_name.push_str(t_name.as_str())
    } else {
        let defaulted = &default_database_table_name_from_entity_name(&macro_data.ty.to_string());
        final_table_name.push_str(defaulted)
    }

    Ok(final_table_name)
}

fn parse_canyon_entity_attr(
    attr: &Attribute,
    schema: &mut Option<String>,
    table_name: &mut Option<String>,
) -> Result<(), TokenStream> {
    if attr
        .path
        .segments
        .iter()
        .any(|seg| seg.ident == "canyon_macros" || seg.ident == "canyon_entity")
    {
        let name_values_result: Result<Punctuated<MetaNameValue, Token![,]>, syn::Error> =
            attr.parse_args_with(Punctuated::parse_terminated);

        if let Ok(meta_name_values) = name_values_result {
            for nv in meta_name_values {
                let ident = nv.path.get_ident();
                if let Some(i) = ident {
                    let identifier = i;
                    match &nv.lit {
                        syn::Lit::Str(s) => {
                            if identifier == "table_name" {
                                *table_name = Some(s.value());
                            } else if identifier == "schema" {
                                *schema = Some(s.value());
                            } else {
                                return Err(
                                    syn::Error::new_spanned(
                                        Ident::new(&identifier.to_string(), i.span()),
                                        "Only string literals are valid values for the attribute arguments"
                                    ).into_compile_error()
                                );
                            }
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                Ident::new(&identifier.to_string(), i.span()),
                                "Only string literals are valid values for the attribute arguments",
                            )
                            .into_compile_error());
                        }
                    }
                } else {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "Only string literals are valid values for the attribute arguments",
                    )
                    .into_compile_error());
                }
            }
        }
    }

    Ok(())
}

/// Autogenerates a default table name for an entity given their struct name
pub fn default_database_table_name_from_entity_name(ty: &str) -> String {
    let mut table_name: String = String::new();

    let mut index = 0;
    for char in ty.chars() {
        if index < 1 {
            table_name.push(char.to_ascii_lowercase());
            index += 1;
        } else {
            match char {
                n if n.is_ascii_uppercase() => {
                    table_name.push('_');
                    table_name.push(n.to_ascii_lowercase());
                }
                _ => table_name.push(char),
            }
        }
    }

    table_name
}

/// Parses the content of an &str to get the related identifier of a type
pub fn database_table_name_to_struct_ident(name: &str) -> Ident {
    let mut struct_name: String = String::new();

    let mut first_iteration = true;
    let mut previous_was_underscore = false;

    for char in name.chars() {
        if first_iteration {
            struct_name.push(char.to_ascii_uppercase());
            first_iteration = false;
        } else {
            match char {
                '_' => {
                    previous_was_underscore = true;
                }
                char if char.is_ascii_lowercase() => {
                    if previous_was_underscore {
                        struct_name.push(char.to_ascii_lowercase())
                    } else {
                        struct_name.push(char)
                    }
                }
                _ => panic!("Detected wrong format or broken convention for database table names"),
            }
        }
    }

    Ident::new(&struct_name, Span::call_site())
}

/// Parses a syn::Identifier to create a defaulted snake case database table name
#[test]
#[cfg(not(target_env = "msvc"))]
fn test_entity_database_name_defaulter() {
    assert_eq!(
        default_database_table_name_from_entity_name("League"),
        "league".to_owned()
    );
    assert_eq!(
        default_database_table_name_from_entity_name("MajorLeague"),
        "major_league".to_owned()
    );
    assert_eq!(
        default_database_table_name_from_entity_name("MajorLeagueTournament"),
        "major_league_tournament".to_owned()
    );

    assert_ne!(
        default_database_table_name_from_entity_name("MajorLeague"),
        "majorleague".to_owned()
    );
    assert_ne!(
        default_database_table_name_from_entity_name("MajorLeague"),
        "MajorLeague".to_owned()
    );
}
