use std::convert::TryFrom;

use proc_macro2::Ident;
use syn::{parse::{Parse, ParseBuffer}, ItemStruct};

use super::entity_fields::EntityField;

/// Provides a convenient way of handling the data on any
/// `CanyonEntity` struct anntotaded with the macro `#[canyon_entity]`
pub struct CanyonEntity {
    pub struct_name: Ident,
    pub attributes: Vec<EntityField>
}

impl Parse for CanyonEntity {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let _struct = input.parse::<ItemStruct>()?;

        let mut parsed_fields = Vec::new();
        for field in _struct.fields {
            let struct_attribute = EntityField::try_from(&field)?;
            parsed_fields.push(struct_attribute);
        }

        Ok(
            Self {
                struct_name: _struct.ident,
                attributes: parsed_fields
            }
        )
    }
}
