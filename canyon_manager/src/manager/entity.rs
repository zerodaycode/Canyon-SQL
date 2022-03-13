use std::convert::TryFrom;
use quote::quote;
use proc_macro2::{Ident, TokenStream};
use syn::{parse::{Parse, ParseBuffer}, ItemStruct, Visibility, Generics};

use super::entity_fields::EntityField;

/// Provides a convenient way of handling the data on any
/// `CanyonEntity` struct anntotaded with the macro `#[canyon_entity]`
pub struct CanyonEntity {
    pub struct_name: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub attributes: Vec<EntityField>
}

use proc_macro2::{Spacing, Punct};
use quote::{TokenStreamExt, ToTokens};


impl ToTokens for CanyonEntity {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for (i, attribute) in self.attributes.iter().enumerate() {
            self.struct_name.to_tokens(tokens);
            self.vis.to_tokens(tokens);
            self.generics.to_tokens(tokens);
            if i > 0 {
                // Double colon `::`
                tokens.append(Punct::new(':', Spacing::Joint));
                tokens.append(Punct::new(':', Spacing::Alone));
            }
            attribute.to_tokens(tokens);
        }
    }
}

impl CanyonEntity {
    pub fn get_attrs_as_token_stream(&self) -> Vec<TokenStream> {
        self
        .attributes
        .iter()
        .map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            quote!{ pub #name: #ty }
        })
        .collect::<Vec<_>>()
    }
}

impl Parse for CanyonEntity {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let _struct = input.parse::<ItemStruct>()?;

        // Retrieve the struct's visibility
        let _vis = _struct.vis;

        // Retrieve the generics attached to this struct
        let _generics = _struct.generics;

        // Retrive the struct fields
        let mut parsed_fields = Vec::new();
        for field in _struct.fields {
            let struct_attribute = EntityField::try_from(&field)?;
            parsed_fields.push(struct_attribute);
        }

        Ok(
            Self {
                struct_name: _struct.ident,
                vis: _vis,
                generics: _generics,
                attributes: parsed_fields
            }
        )
    }
}
