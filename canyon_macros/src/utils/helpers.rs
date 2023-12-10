use proc_macro2::{Ident, Span, TokenStream};
use syn::{punctuated::Punctuated, MetaNameValue, Token};

use super::macro_tokens::MacroTokens;

/// If the `canyon_entity` macro has valid attributes attached, and those attrs are the
/// user's desired `table_name` and/or the `schema_name`, this method returns its
/// correct form to be wired as the table name that the CRUD methods requires for generate
/// the queries
pub fn table_schema_parser(macro_data: &MacroTokens<'_>) -> Result<String, TokenStream> {
    let mut table_name: Option<String> = None;
    let mut schema: Option<String> = None;

    for attr in macro_data.attrs {
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
                                    table_name = Some(s.value())
                                } else if identifier == "schema" {
                                    schema = Some(s.value())
                                } else {
                                    return Err(
                                        syn::Error::new_spanned(
                                            Ident::new(&identifier.to_string(), i.span()),
                                            "Only string literals are valid values for the attribute arguments"
                                        ).into_compile_error()
                                    );
                                }
                            }
                            _ => return Err(syn::Error::new_spanned(
                                Ident::new(&identifier.to_string(), i.span()),
                                "Only string literals are valid values for the attribute arguments",
                            )
                            .into_compile_error()),
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

            let mut final_table_name = String::new();
            if schema.is_some() {
                final_table_name.push_str(format!("{}.", schema.unwrap()).as_str())
            }

            if let Some(t_name) = table_name {
                final_table_name.push_str(t_name.as_str())
            } else {
                let defaulted =
                    &default_database_table_name_from_entity_name(&macro_data.ty.to_string());
                final_table_name.push_str(defaulted)
            }

            return Ok(final_table_name);
        }
    }

    Ok(macro_data.ty.to_string())
}

/// Parses a syn::Identifier to get a snake case database name from the type identifier
pub fn _database_table_name_from_struct(ty: &Ident) -> String {
    let struct_name: String = ty.to_string();
    let mut table_name: String = String::new();

    let mut index = 0;
    for char in struct_name.chars() {
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
///
pub fn default_database_table_name_from_entity_name(ty: &str) -> String {
    let struct_name: String = ty.to_string();
    let mut table_name: String = String::new();

    let mut index = 0;
    for char in struct_name.chars() {
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

    Ident::new(&struct_name, proc_macro2::Span::call_site())
}
