use std::convert::TryFrom;

use proc_macro2::Ident;
use quote::ToTokens;
use syn::{Type, Attribute, Field};

use super::field_annotation::EntityFieldAnnotation;

/// Represents any of the fields and annotations (if any valid annotation) found for a CanyonEntity
pub struct EntityField {
    pub name: Ident,
    pub attribute_type: EntityFieldAnnotation,
    pub ty: Type,
}

impl EntityField {
    pub fn new(name: &Ident, raw_helper_attributes: &[Attribute], ty: &Type) -> syn::Result<Self> {
        // Getting the name of attributes put in front of struct fields
        let helper_attributes = raw_helper_attributes
            .iter()
            .map(|attribute| {
                attribute
                    .path
                    .segments
                    .iter()
                    .map(|segment| &segment.ident)
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        // Making sense of the attribute(s)
        let attribute_type = if helper_attributes.is_empty() {
            EntityFieldAnnotation::ForeignKey
        } else if helper_attributes.len() == 1 {
            let helper_attribute = helper_attributes[0];
            EntityFieldAnnotation::try_from(helper_attribute)?
        } else {
            return Err(
                syn::Error::new_spanned(
                    name, 
                    "Field has more than one attribute"
                )
            );
        };

        Ok(
            Self {
                name: name.clone(),
                ty: ty.clone(),
                attribute_type,
            }
        )
    }
}

impl TryFrom<&Field> for EntityField {
    type Error = syn::Error;

    fn try_from(field: &Field) -> Result<Self, Self::Error> {
        let name = field
            .ident
            .as_ref()
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    field.to_token_stream(), 
                    "Expected a structure with named fields, unnamed field given"
                )
            })?;


        Self::new(&name, &field.attrs, &field.ty)
    }
}
