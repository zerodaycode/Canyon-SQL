use std::convert::TryFrom;
use partialdebug::placeholder::PartialDebug;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::{Type, Attribute, Field};

use super::field_annotation::EntityFieldAnnotation;
/// Represents any of the fields and annotations (if any valid annotation) found for a CanyonEntity
#[derive(PartialDebug, Clone)]
pub struct EntityField {
    pub name: Ident,
    pub field_type: Type,
    pub attribute: Option<EntityFieldAnnotation>,
}

impl EntityField {
    pub fn get_field_type_as_string(&self) -> String {
        match &self.field_type {
            Type::Array(type_) => type_.to_token_stream().to_string(),
            Type::BareFn(type_) => type_.to_token_stream().to_string(),
            Type::Group(type_) => type_.to_token_stream().to_string(),
            Type::ImplTrait(type_) => type_.to_token_stream().to_string(),
            Type::Infer(type_) => type_.to_token_stream().to_string(),
            Type::Macro(type_) => type_.to_token_stream().to_string(),
            Type::Never(type_) => type_.to_token_stream().to_string(),
            Type::Paren(type_) => type_.to_token_stream().to_string(),
            Type::Path(type_) => type_.to_token_stream().to_string(),
            Type::Ptr(type_) => type_.to_token_stream().to_string(),
            Type::Reference(type_) => type_.to_token_stream().to_string(),
            Type::Slice(type_) => type_.to_token_stream().to_string(),
            Type::TraitObject(type_) => type_.to_token_stream().to_string(),
            Type::Tuple(type_) => type_.to_token_stream().to_string(),
            Type::Verbatim(type_) => type_.to_token_stream().to_string(),
            _ => "".to_owned(),
        }
    }


    pub fn new(name: &Ident, raw_helper_attributes: &[Attribute], ty: &Type) -> syn::Result<Self> {
        // Getting the name of attributes put in front of struct fields
        let helper_attributes = raw_helper_attributes
            .iter()
            .map(|attribute| { attribute })
            .collect::<Vec<_>>();
            
        // Making sense of the attribute(s)
        let attribute_type = if helper_attributes.len() == 1 {
            let helper_attribute = &helper_attributes[0];
            Some(EntityFieldAnnotation::try_from(helper_attribute)?)

        } else if helper_attributes.len() > 1 {
            return Err(
                syn::Error::new_spanned(
                    name, 
                    "Field has more than one attribute"
                )
            );
        } else { None };

        Ok(
            Self {
                name: name.clone(),
                field_type: ty.clone(),
                attribute: attribute_type,
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
