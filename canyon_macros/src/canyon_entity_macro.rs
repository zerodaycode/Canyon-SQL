use proc_macro2::{Span, TokenStream};
use syn::NestedMeta;

pub(crate) fn parse_canyon_entity_proc_macro_attr(
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
            syn::NestedMeta::Meta(m) => {
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
            syn::NestedMeta::Lit(_) => {
                parsing_attribute_error = Some(syn::Error::new(
                    Span::call_site(),
                    "No literal values allowed on the `canyon_macros::canyon_entity` proc macro"
                ).into_compile_error());
            }
        }
    }

    (table_name, schema_name, parsing_attribute_error)
}
