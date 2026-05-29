use crate::utils::helpers;
use canyon_entities::CANYON_REGISTER_ENTITIES;
use canyon_entities::entity::CanyonEntity;
use canyon_entities::entity_fields::EntityField;
use canyon_entities::manager_builder::generate_user_struct;
use canyon_entities::register_types::{CanyonRegisterEntity, CanyonRegisterEntityField};
use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Expr, Lit, Meta, Token};

pub type CanyonEntityAttributeArgs = Punctuated<Meta, Token![,]>;

pub fn generate_canyon_entity_tokens(
    attrs: CanyonEntityAttributeArgs,
    input: CompilerTokenStream,
) -> TokenStream {
    let parsed_attrs = parse_canyon_entity_proc_macro_attr(attrs);

    let entity = match syn::parse::<CanyonEntity>(input) {
        Ok(entity) => entity,
        Err(error) => return error.into_compile_error(),
    };

    let generated_user_struct = generate_user_struct(&entity);
    let register_entity =
        build_register_entity(&entity, parsed_attrs.table_name, parsed_attrs.schema_name);

    CANYON_REGISTER_ENTITIES
        .lock()
        .expect("Error acquiring Mutex guard on Canyon Entity macro")
        .push(register_entity);

    if let Some(error) = parsed_attrs.error {
        quote! {
            #error
            #generated_user_struct
        }
    } else {
        quote! {
            #generated_user_struct
        }
    }
}

fn build_register_entity<'a>(
    entity: &CanyonEntity,
    table_name: Option<&'static str>,
    schema_name: Option<&'static str>,
) -> CanyonRegisterEntity<'a> {
    let entity_name = leak_string(entity.struct_name.to_string());

    CanyonRegisterEntity {
        entity_name,
        entity_db_table_name: table_name.unwrap_or_else(|| {
            leak_string(helpers::default_database_table_name_from_entity_name(
                entity_name,
            ))
        }),
        user_schema_name: schema_name,
        entity_fields: entity
            .fields
            .iter()
            .map(build_register_entity_field)
            .collect(),
    }
}

fn build_register_entity_field(field: &EntityField) -> CanyonRegisterEntityField {
    CanyonRegisterEntityField {
        field_name: field.name.to_string(),
        field_type: field.get_field_type_as_string().replace(' ', ""),
        annotations: field
            .attributes
            .iter()
            .map(|attr| attr.get_as_string())
            .collect(),
        ..Default::default()
    }
}

#[derive(Default)]
struct ParsedCanyonEntityAttrs {
    table_name: Option<&'static str>,
    schema_name: Option<&'static str>,
    error: Option<TokenStream>,
}

fn parse_canyon_entity_proc_macro_attr(
    attrs: CanyonEntityAttributeArgs,
) -> ParsedCanyonEntityAttrs {
    let mut parsed = ParsedCanyonEntityAttrs::default();

    for meta in attrs {
        if let Err(error) = parse_canyon_entity_meta(meta, &mut parsed) {
            parsed.error = Some(error.into_compile_error());
        }
    }

    parsed
}

fn parse_canyon_entity_meta(meta: Meta, parsed: &mut ParsedCanyonEntityAttrs) -> syn::Result<()> {
    let Meta::NameValue(name_value) = meta else {
        return Err(syn::Error::new(
            Span::call_site(),
            "Only argument identifiers with a value after an `=` sign are allowed on the `canyon_macros::canyon_entity` proc macro",
        ));
    };

    let ident = name_value.path.get_ident().ok_or_else(|| {
        syn::Error::new_spanned(
            &name_value.path,
            "Only simple identifiers are valid keys for `canyon_entity` attribute arguments",
        )
    })?;

    let value = parse_string_literal(&name_value.value)?;

    match ident.to_string().as_str() {
        "table_name" => parsed.table_name = Some(leak_string(value)),
        "schema" => parsed.schema_name = Some(leak_string(value)),
        _ => {
            return Err(syn::Error::new_spanned(
                ident,
                format!("Argument `{ident}` is not allowed in the `canyon_entity` macro attribute"),
            ));
        }
    }

    Ok(())
}

fn parse_string_literal(expr: &Expr) -> syn::Result<String> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(value) => Ok(value.value()),
            _ => Err(syn::Error::new_spanned(
                expr,
                "Only string literals are valid values for the attributes",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            expr,
            "Only literal expressions are valid values for the attributes",
        )),
    }
}

fn leak_string(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}
