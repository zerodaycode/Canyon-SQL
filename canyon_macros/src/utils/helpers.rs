use super::macro_tokens::MacroTokens;
use canyon_core::query::querybuilder::syntax::table_metadata::TableMetadata;
pub(crate) use canyon_entities::helpers::default_database_table_name_from_entity_name;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::borrow::Cow;
use std::fmt::Write;
use syn::{Attribute, Field, Fields, Type, TypeGenerics, Visibility};

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

pub fn placeholders_generator(num_values: usize) -> String {
    let mut placeholders = String::new();
    for (i, n) in (1..num_values).enumerate() {
        if i > 0 {
            placeholders.push_str(", ");
        }
        write!(placeholders, "${}", n).unwrap();
    }

    placeholders
}

pub fn field_has_target_attribute(field: &Field, target_attribute: &str) -> bool {
    field.attrs.iter().any(|attr| {
        attr.path()
            .segments
            .first()
            .map(|segment| segment.ident == target_attribute)
            .unwrap_or(false)
    })
}

/// If the `canyon_entity` macro has valid attributes attached, and those attrs are the
/// user's desired `table_name` and/or the `schema_name`, this method returns its
/// correct form to be wired as the table name that the CRUD methods requires for generate
/// the queries
pub fn table_schema_parser<'a>(
    macro_data: &MacroTokens<'_>,
) -> Result<TableMetadata<'a>, TokenStream> {
    let mut table_name: Option<Cow<'_, str>> = None;
    let mut schema: Option<Cow<'_, str>> = None;

    for attr in macro_data.attrs {
        if __impl::is_canyon_entity_attr(attr) {
            parse_canyon_entity_attr(attr, &mut schema, &mut table_name)?;
        }
    }

    let mut table_meta = TableMetadata::default();
    if let Some(schema_) = schema {
        table_meta.schema(schema_);
    }

    if let Some(t_name) = table_name {
        table_meta.table_name(t_name);
    } else {
        let target_type = if let Some(mapper_ty) = macro_data.retrieve_mapping_target_type() {
            mapper_ty.to_string()
        } else {
            macro_data.ty.to_string()
        };
        table_meta.table_name(default_database_table_name_from_entity_name(&target_type));
    }

    Ok(table_meta)
}

fn parse_canyon_entity_attr(
    attr: &Attribute,
    schema: &mut Option<Cow<'_, str>>,
    table_name: &mut Option<Cow<'_, str>>,
) -> Result<(), TokenStream> {
    for name_value in __impl::parse_canyon_entity_args(attr)? {
        let key = __impl::name_value_key(&name_value)?;
        let value = __impl::string_literal_value(&name_value)?;

        if key == "schema" {
            *schema = Some(Cow::Owned(value));
        } else if key == "table_name" {
            *table_name = Some(Cow::Owned(value));
        } else {
            return Err(__impl::unknown_canyon_entity_arg(&name_value));
        }
    }

    Ok(())
}

mod __impl {
    use proc_macro2::TokenStream;
    use syn::{Attribute, Expr, Lit, Meta, MetaNameValue, Token, punctuated::Punctuated};

    pub(super) fn is_canyon_entity_attr(attr: &Attribute) -> bool {
        attr.path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "canyon_entity")
    }

    pub(super) fn parse_canyon_entity_args(
        attr: &Attribute,
    ) -> Result<Punctuated<MetaNameValue, Token![,]>, TokenStream> {
        match &attr.meta {
            Meta::Path(_) => Ok(Punctuated::new()),
            Meta::List(_) => attr
                .parse_args_with(Punctuated::parse_terminated)
                .map_err(syn::Error::into_compile_error),
            Meta::NameValue(_) => Err(syn::Error::new_spanned(
                &attr.meta,
                "`canyon_entity` attribute expects a list of arguments",
            )
            .into_compile_error()),
        }
    }

    pub(super) fn name_value_key(name_value: &MetaNameValue) -> Result<&syn::Ident, TokenStream> {
        name_value.path.get_ident().ok_or_else(|| {
            syn::Error::new_spanned(
                &name_value.path,
                "Only simple identifiers are valid keys for `canyon_entity` attribute arguments",
            )
            .into_compile_error()
        })
    }

    pub(super) fn string_literal_value(name_value: &MetaNameValue) -> Result<String, TokenStream> {
        match &name_value.value {
            Expr::Lit(expr_lit) => match &expr_lit.lit {
                Lit::Str(value) => Ok(value.value()),
                _ => Err(syn::Error::new_spanned(
                    &name_value.value,
                    "Only string literals are valid values for `canyon_entity` attribute arguments",
                )
                .into_compile_error()),
            },
            _ => Err(syn::Error::new_spanned(
                &name_value.value,
                "Only literal expressions are valid values for `canyon_entity` attribute arguments",
            )
            .into_compile_error()),
        }
    }

    pub(super) fn unknown_canyon_entity_arg(name_value: &MetaNameValue) -> TokenStream {
        syn::Error::new_spanned(
            &name_value.path,
            "Only `table_name` and `schema` are valid `canyon_entity` attribute arguments",
        )
        .into_compile_error()
    }
}

#[cfg(test)]
mod tests_for_parse_struct_field_attributes {
    use super::*;
    use syn::{ItemStruct, parse_str};

    #[test]
    fn detects_target_attribute_correctly() {
        let input = r#"
            struct Test {
                #[my_attr]
                field1: String,
                field2: i32,
            }
        "#;

        // Parse the struct
        let item: ItemStruct = parse_str(input).expect("Failed to parse struct");
        let fields: Vec<_> = item.fields.iter().collect();

        // Check the field with #[my_attr]
        assert!(field_has_target_attribute(fields[0], "my_attr"));
        // Check the field without the attribute
        assert!(!field_has_target_attribute(fields[1], "my_attr"));
    }

    #[test]
    fn parses_canyon_entity_table_name_and_schema() {
        let input: syn::DeriveInput = parse_str(
            r#"
            #[canyon_entity(table_name = "users", schema = "public")]
            struct User;
            "#,
        )
        .expect("failed to parse derive input");

        let mut schema = None;
        let mut table_name = None;

        parse_canyon_entity_attr(&input.attrs[0], &mut schema, &mut table_name)
            .expect("failed to parse canyon_entity attribute");

        assert_eq!(table_name.as_deref(), Some("users"));
        assert_eq!(schema.as_deref(), Some("public"));
    }

    #[test]
    fn rejects_unknown_canyon_entity_attribute_keys() {
        let input: syn::DeriveInput = parse_str(
            r#"
            #[canyon_entity(foo = "bar")]
            struct User;
            "#,
        )
        .expect("failed to parse derive input");

        let mut schema = None;
        let mut table_name = None;

        let err = parse_canyon_entity_attr(&input.attrs[0], &mut schema, &mut table_name)
            .expect_err("unknown canyon_entity keys must fail");

        assert!(err.to_string().contains("compile_error"));
        assert_eq!(table_name, None);
        assert_eq!(schema, None);
    }

    #[test]
    fn rejects_non_string_canyon_entity_attribute_values() {
        let input: syn::DeriveInput = parse_str(
            r#"
            #[canyon_entity(table_name = 42)]
            struct User;
            "#,
        )
        .expect("failed to parse derive input");

        let mut schema = None;
        let mut table_name = None;

        let err = parse_canyon_entity_attr(&input.attrs[0], &mut schema, &mut table_name)
            .expect_err("non-string canyon_entity values must fail");

        assert!(err.to_string().contains("compile_error"));
        assert_eq!(table_name, None);
        assert_eq!(schema, None);
    }
}
