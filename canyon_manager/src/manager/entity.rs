use std::convert::TryFrom;
use proc_macro2::{Ident, TokenStream};
use syn::{parse::{Parse, ParseBuffer}, ItemStruct, Visibility, Generics};
use quote::{quote};
use partialdebug::placeholder::PartialDebug;

use super::entity_fields::EntityField;

/// Provides a convenient way of handling the data on any
/// `CanyonEntity` struct anntotaded with the macro `#[canyon_entity]`
#[derive(PartialDebug)]
pub struct CanyonEntity {
    pub struct_name: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub attributes: Vec<EntityField>
}

impl CanyonEntity {
    pub fn get_entity_as_string(&self) -> String {
        let mut as_string = String::new();
        as_string.push_str("Identifier -> ");
        as_string.push_str(self.struct_name.to_string().as_str());
        as_string.push_str("; Columns -> ");
        as_string.push_str(self.get_attrs_as_string().as_str());

        println!("String of register: {:?}", as_string);
        as_string
    }


    fn get_attrs_as_string(&self) -> String {
        let mut vec_columns = Vec::new();
        for attribute in self.attributes.iter() {
            let name = attribute.name.to_string();
            let field_type = attribute.get_field_type_as_string();
            let column_name_type_tuple = format!("({}:{})", name, field_type);
            vec_columns.push(column_name_type_tuple)
        }

        let columns_str = vec_columns.join(",");

        columns_str

    }

    pub fn get_attrs_as_token_stream(&self) -> Vec<TokenStream> {
        self.attributes
            .iter()
            .map(|f| {
                let name = &f.name;
                let ty = &f.field_type;
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
