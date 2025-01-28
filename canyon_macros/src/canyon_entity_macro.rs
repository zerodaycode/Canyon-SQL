use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{AttributeArgs, NestedMeta};
use canyon_entities::CANYON_REGISTER_ENTITIES;
use canyon_entities::entity::CanyonEntity;
use canyon_entities::manager_builder::generate_user_struct;
use canyon_entities::register_types::{CanyonRegisterEntity, CanyonRegisterEntityField};
use crate::utils::helpers;

pub fn generate_canyon_entity_tokens(attrs: AttributeArgs, input: CompilerTokenStream) -> TokenStream {
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

fn parse_canyon_entity_proc_macro_attr(
    attrs: Vec<NestedMeta>,
) -> (
    Option<&'static str>,
    Option<&'static str>,
    Option<TokenStream>,
) {
    let mut table_name: Option<&str> = None;
    let mut schema_name: Option<&str> = None;

    let mut parsing_attribute_error: Option<TokenStream> = None;

    // The parse of the available options to configure the Canyon Entity
    for element in attrs {
        match element {
            NestedMeta::Meta(m) => {
                match m {
                    syn::Meta::NameValue(nv) => {
                        let attr_arg_ident = nv
                            .path
                            .get_ident()
                            .expect("Something went wrong parsing the `table_name` argument")
                            .to_string();

                        if &attr_arg_ident == "table_name" || &attr_arg_ident == "schema" {
                            match nv.lit {
                                syn::Lit::Str(ref l) => {
                                    if &attr_arg_ident == "table_name" {
                                        table_name = Some(Box::leak(l.value().into_boxed_str()))
                                    } else {
                                        schema_name = Some(Box::leak(l.value().into_boxed_str()))
                                    }
                                }
                                _ => {
                                    parsing_attribute_error = Some(syn::Error::new(
                                        Span::call_site(),
                                        "Only string literals are valid values for the attributes"
                                    ).into_compile_error());
                                }
                            }
                        } else {
                            parsing_attribute_error = Some(
                                syn::Error::new(
                                    Span::call_site(),
                                    format!(
                                        "Argument: `{:?}` are not allowed in the canyon_macro attr",
                                        &attr_arg_ident
                                    ),
                                )
                                .into_compile_error(),
                            );
                        }
                    }
                    _ => {
                        parsing_attribute_error = Some(syn::Error::new(
                            Span::call_site(),
                            "Only argument identifiers with a value after an `=` sign are allowed on the `canyon_macros::canyon_entity` proc macro"
                        ).into_compile_error());
                    }
                }
            }
            NestedMeta::Lit(_) => {
                parsing_attribute_error = Some(syn::Error::new(
                    Span::call_site(),
                    "No literal values allowed on the `canyon_macros::canyon_entity` proc macro"
                ).into_compile_error());
            }
        }
    }

    (table_name, schema_name, parsing_attribute_error)
}
