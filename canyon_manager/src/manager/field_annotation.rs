use std::{convert::TryFrom, collections::HashMap};
use proc_macro2::Ident;
use syn::{Attribute, Token, punctuated::Punctuated, MetaNameValue};

/// The available annotations for a field that belongs to any struct
/// annotaded with `#[canyon_entity]`
#[derive(Debug, Clone)]
pub enum EntityFieldAnnotation {
    PrimaryKey(bool),
    ForeignKey(String, String)
}

impl EntityFieldAnnotation {

    /// Returns the data of the [`EntityFieldAnnotation`] in a understandable format for `Canyon`
    pub fn get_as_string(&self) -> String {
        match &*self {
            Self::PrimaryKey(autoincremental) => 
                format!("Annotation: PrimaryKey, Autoincremental: {}", autoincremental),
            Self::ForeignKey(table, column) => 
                format!("Annotation: ForeignKey, Table: {}, Column: {}", table, column),
        }
    }

    fn primary_key_parser(ident: &Ident, attr_args: &Result<Punctuated<MetaNameValue, Token![,]>, syn::Error>) -> syn::Result<Self> {
        match attr_args {
            Ok(name_value) => {
                let mut data: HashMap<String, bool> = HashMap::new();
                for nv in name_value {
                    // The identifier
                    let attr_value_ident = nv.path.get_ident().unwrap().to_string();
                    // The value after the Token[=]
                    let attr_value = match &nv.lit {
                        // Error if the token is not a boolean literal
                        syn::Lit::Bool(v) => v.value(),
                        _ => {
                            return Err(
                                syn::Error::new_spanned(
                                    nv.path.clone(), 
                                    format!("Only bool literals are supported for the `{}` attribute", &attr_value_ident)
                                )
                            )
                        }
                    };
                    data.insert(attr_value_ident, attr_value);
                }

                Ok(
                    EntityFieldAnnotation::PrimaryKey(
                        match data.get("autoincremental") {
                            Some(aut) => aut.to_owned(),
                            None => {  // TODO En vez de error, false para default
                                return Err(
                                    syn::Error::new_spanned(
                                        ident, 
                                        "Missed `autoincremental` argument on the Primary Key annotation".to_string()
                                    )
                                )
                            },
                        }, 
                    )
                )
            },
            Err(_) => Ok(
                EntityFieldAnnotation::PrimaryKey(true)
            )
        }
    }

    fn foreign_key_parser(ident: &Ident, attr_args: &Result<Punctuated<MetaNameValue, Token![,]>, syn::Error>) -> syn::Result<Self> {
        match attr_args {
            Ok(name_value) => {
                let mut data: HashMap<String, String> = HashMap::new();

                for nv in name_value {
                    // The identifier
                    let attr_value_ident = nv.path.get_ident().unwrap().to_string();
                    // The value after the Token[=]
                    let attr_value = match &nv.lit {
                        // Error if the token is not a string literal
                        // TODO Implement the option (or change it to) to use a Rust Ident instead a Str Lit
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
                                        ident, 
                                        "Missed `table` argument on the Foreign Key annotation".to_string()
                                    )
                                )
                            },
                        }, 
                        match data.get("column") {
                            Some(table) => table.to_owned(),
                            None => {
                                return Err(
                                    syn::Error::new_spanned(
                                        ident, 
                                        "Missed `column` argument on the Foreign Key annotation".to_string()
                                    )
                                )
                            },
                        }, 
                    )
                )
            },
            Err(_) => return Err(
                syn::Error::new_spanned(
                    ident, 
                    "Error generating the Foreign Key".to_string()
                )
            ),
        }
    }
}


impl TryFrom<&&Attribute> for EntityFieldAnnotation {
    type Error = syn::Error;

    fn try_from(attribute: &&Attribute) -> Result<Self, Self::Error> {
        let ident = attribute.path.segments[0].ident.clone();
        let name_values: Result<Punctuated<MetaNameValue, Token![,]>, syn::Error> = 
            attribute.parse_args_with(Punctuated::parse_terminated);

        Ok(
            match ident.clone().to_string().as_str() {
                "primary_key" => 
                    EntityFieldAnnotation::primary_key_parser(&ident, &name_values)?,
                "foreign_key" => 
                    EntityFieldAnnotation::foreign_key_parser(&ident, &name_values)?,
                _ => {
                    return Err(
                        syn::Error::new_spanned(
                            ident.clone(), 
                            format!("Unknown attribute `{}`", &ident)
                        )
                    )
                }
            }
        )
    }
}