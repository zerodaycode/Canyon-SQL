use std::{convert::TryFrom, collections::HashMap};
use proc_macro2::{Ident, Span};
use syn::{Attribute, Token, punctuated::Punctuated, MetaNameValue};

/// The available annotations for a field that belongs to any struct
/// annotaded with `#[canyon_entity]`
#[derive(Debug, Clone)]
pub enum EntityFieldAnnotation {
    ForeignKey(String, String)
}

impl EntityFieldAnnotation {

    pub fn new(attr_args: Result<Punctuated<MetaNameValue, Token![,]>, syn::Error>) -> Result<Self, syn::Error> {
        Self::foreign_key_parser(attr_args)
    }

    pub fn get_as_string(&self) -> String {
        match &*self {
            Self::ForeignKey(table, column) => 
                format!("Table: {}, Column: {}", table, column)
        }
    }

    pub fn foreign_key_parser(attr_args: Result<Punctuated<MetaNameValue, Token![,]>, syn::Error>) -> Result<Self, syn::Error> {
        match attr_args {
            Ok(name_value) => {
                let mut data: HashMap<String, String> = HashMap::new();

                for nv in name_value {
                    // The identifier
                    let attr_value_ident = nv.path.get_ident().unwrap().to_string();
                    // The value after the Token[=]
                    let attr_value = match nv.lit {
                        // Error if the token is not a string literal
                        syn::Lit::Str(v) => v.value(),
                        _ => {
                            return Err(
                                syn::Error::new_spanned(
                                    nv.path.clone(), 
                                    format!("Only string literals are supported for the `{}` attribute", attr_value_ident)
                                )
                            )
                        }
                    };
                    data.insert(attr_value_ident, attr_value);
                }

                Ok(
                    EntityFieldAnnotation::ForeignKey(
                        match data.get("table") {
                            Some(table) => table.to_owned(),
                            None => {
                                return Err(
                                    syn::Error::new_spanned(
                                            Ident::new("Table", Span::call_site()), 
                                        format!("Missed `table` argument on the Foreign Key annotation")
                                    )
                                )
                            },
                        }, 
                        match data.get("column") {
                            Some(table) => table.to_owned(),
                            None => {
                                return Err(
                                    syn::Error::new_spanned(
                                            Ident::new("Column", Span::call_site()), 
                                        format!("Missed `column` argument on the Foreign Key annotation")
                                    )
                                )
                            },
                        }, 
                    )
                )
            },
            Err(_) => return Err(
                syn::Error::new(
                        Span::call_site(), 
                    format!("Error generating the Foreign Key")
                )
            ),
        }
    }
}


impl TryFrom<&&Attribute> for EntityFieldAnnotation {
    type Error = syn::Error;

    fn try_from(attribute: &&Attribute) -> Result<Self, Self::Error> {

        let name_values: Result<Punctuated<MetaNameValue, Token![,]>, syn::Error> = 
            attribute.parse_args_with(Punctuated::parse_terminated);


        let ident = attribute.path.segments[0].ident.clone();
        Ok(
            match ident.clone().to_string().as_str() {
                "foreign_key" => 
                    EntityFieldAnnotation::new(name_values)
                        .expect("Error generating the Foreign Key details"),
                _ => {
                    return Err(
                        syn::Error::new_spanned(
                            ident.clone(), 
                            format!("Unknown attribute `{}`", ident)
                        )
                    )
                }
            }
        )
    }
}