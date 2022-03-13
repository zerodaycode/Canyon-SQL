use std::convert::TryFrom;
use proc_macro2::{Spacing, Span, Punct, TokenStream};
use quote::{TokenStreamExt, ToTokens};
use proc_macro2::Ident;

/// The available annotations for a field that belongs to any struct
/// annotaded with `#[canyon_entity]`
#[derive(Debug)]
pub enum EntityFieldAnnotation {
    ForeignKey
}

impl ToTokens for EntityFieldAnnotation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let enum_ident = Ident::new("EntityFieldAnnotation", Span::call_site());
        tokens.append(enum_ident.clone());
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));

        match *self {
            EntityFieldAnnotation::ForeignKey => tokens.append(enum_ident.clone()),
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