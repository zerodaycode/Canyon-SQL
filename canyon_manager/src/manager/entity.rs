use std::convert::TryFrom;
use proc_macro2::{Ident, TokenStream};
use syn::{parse::{Parse, ParseBuffer}, ItemStruct, Visibility, Generics};
use quote::{quote};
use partialdebug::placeholder::PartialDebug;

use super::entity_fields::EntityField;

/// Provides a convenient way of handling the data on any
/// `CanyonEntity` struct anntotaded with the macro `#[canyon_entity]`
#[derive(PartialDebug, Clone)]
pub struct CanyonEntity {
    pub struct_name: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub attributes: Vec<EntityField>
}

unsafe impl Send for CanyonEntity {}
unsafe impl Sync for CanyonEntity {}

impl CanyonEntity {
    /// Creates an enum with the names of the fields as the variants of the type,
    /// where the enum type corresponds with the struct's type that belongs
    /// + a concatenation of "Fields" after it
    pub fn get_fields_as_enum_variants(&self) -> Vec<TokenStream> {
        self.attributes
            .iter()
            .map(|f| {
                let field_name = &f.name;
                let ty = &f.field_type;
                quote!{ #field_name(#ty) }
            })
        .collect::<Vec<_>>()
    }

    /// Generates an implementation of the match pattern to find whatever variant
    /// is being requested // TODO Better docs please
    pub fn create_match_arm_for_relate_field(&self, enum_name: &Ident) -> Vec<TokenStream> {
        self.attributes
            .iter()
            .map( |f| {
                let field_name = &f.name;
                let field_name_as_string = f.name.to_string();
                let field_type_as_string = f.get_field_type_as_string();

                let quote = if field_type_as_string.contains("Option") {
                    quote! { 
                        #enum_name::#field_name(v) => 
                            format!("{} {}", #field_name_as_string.to_string(), v.unwrap().to_string())
                    }
                } else {
                    quote! { 
                        #enum_name::#field_name(v) => 
                            format!("{} {}", #field_name_as_string.clone().to_string(), v.to_string())
                    }
                }; 

                quote
            })
        .collect::<Vec<_>>()
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
        let mut parsed_fields: Vec<EntityField> = Vec::new();
        for field in _struct.fields {
            let struct_attribute = EntityField::try_from(&field);
            match struct_attribute {
                Ok(attribute) => {
                    println!("Parsing correctly field: {:?}", attribute.name.to_string());
                    parsed_fields.push(attribute)
                },
                Err(e) => {
                    println!("Error parsing field");
                    return Err(e)
                },
            }
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
