use std::convert::TryFrom;
use proc_macro2::Ident;

/// The available annotations for a field that belongs to any struct
/// annotaded with `#[canyon_entity]`
#[derive(Debug)]
pub enum EntityFieldAnnotation {
    ForeignKey
}

impl EntityFieldAnnotation {
    pub fn get_as_string(&self) -> String {
        match *self {
            Self::ForeignKey => "ForeignKey".to_owned()
        }
    }
}


impl TryFrom<&Ident> for EntityFieldAnnotation {
    type Error = syn::Error;

    fn try_from(ident: &Ident) -> Result<Self, Self::Error> {
        Ok(
            // Idents have a string representation we can use
            match ident.to_string().as_str() {
                "foreign_key" => EntityFieldAnnotation::ForeignKey,
                _ => {
                    return Err(
                        syn::Error::new_spanned(
                            ident, 
                            format!("Unknown attribute `{}`", ident)
                        )
                    )
                }
            }
        )
    }
}