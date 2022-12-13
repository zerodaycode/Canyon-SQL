use partialdebug::placeholder::PartialDebug;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::convert::TryFrom;
use syn::{
    parse::{Parse, ParseBuffer},
    Attribute, Generics, ItemStruct, Visibility,
};

use super::entity_fields::EntityField;

/// Provides a convenient way of handling the data on any
/// `CanyonEntity` struct anntotaded with the macro `#[canyon_entity]`
#[derive(PartialDebug, Clone)]
pub struct CanyonEntity {
    pub struct_name: Ident,
    pub user_table_name: Option<String>,
    pub user_schema_name: Option<String>,
    pub vis: Visibility,
    pub generics: Generics,
    pub fields: Vec<EntityField>,
    pub attrs: Vec<Attribute>,
}

unsafe impl Send for CanyonEntity {}
unsafe impl Sync for CanyonEntity {}

impl CanyonEntity {
    /// Generates as many variants for the enum as fields has the type
    /// which this enum is related to, and that type it's the entity
    /// stored in [`CanyonEntity`]
    /// of the corresponding field
    pub fn get_fields_as_enum_variants(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|f| {
                let field_name = &f.name;
                quote! { #field_name }
            })
            .collect::<Vec<_>>()
    }

    /// Generates as many variants for the enum as fields has the type
    /// which this enum is related to, and that type it's the entity
    /// stored in [`CanyonEntity`]
    ///
    /// Makes a variant `#field_name(#ty)` where `#ty` it's the type
    /// of the corresponding field
    pub fn get_fields_as_enum_variants_with_type(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|f| {
                let field_name = &f.name;
                let ty = &f.field_type;
                quote! { #field_name(#ty) }
            })
            .collect::<Vec<_>>()
    }

    /// Generates an implementation of the match pattern to find whatever variant
    /// is being requested when the method `.field_name_as_str(self)` it's invoked over some
    /// instance that implements the `canyon_sql::crud::bounds::FieldIdentifier` trait
    pub fn create_match_arm_for_get_variant_as_string(
        &self,
        enum_name: &Ident,
    ) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|f| {
                let field_name = &f.name;
                let field_name_as_string = f.name.to_string();

                quote! {
                    #enum_name::#field_name => #field_name_as_string.to_string()
                }
            })
            .collect::<Vec<_>>()
    }

    /// Generates an implementation of the match pattern to find whatever variant
    /// is being requested when the method `.value()` it's invoked over some
    /// instance that implements the `canyon_sql::crud::bounds::FieldValueIdentifier` trait
    pub fn create_match_arm_for_relate_fields_with_values(
        &self,
        enum_name: &Ident,
    ) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map( |f| {
                let field_name = &f.name;
                let field_name_as_string = f.name.to_string();
                let field_type_as_string = f.get_field_type_as_string();
                
                if field_type_as_string.contains("Option") {
                    quote! {
                        #enum_name::#field_name(v) =>
                            format!("{} {}", #field_name_as_string, v.unwrap().to_string())
                    }
                } else {
                    quote! {
                        #enum_name::#field_name(v) =>
                            format!("{} {}", #field_name_as_string, v.to_string())
                    }
                }
            })
        .collect::<Vec<_>>()
    }

    pub fn get_attrs_as_token_stream(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|f| {
                let name = &f.name;
                let ty = &f.field_type;
                quote! { pub #name: #ty }
            })
            .collect::<Vec<_>>()
    }
}

impl Parse for CanyonEntity {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let _struct = input.parse::<ItemStruct>()?;

        // Retrive the struct fields
        let mut parsed_fields: Vec<EntityField> = Vec::new();
        for field in _struct.fields {
            let struct_attribute = EntityField::try_from(&field)?;
            parsed_fields.push(struct_attribute)
        }

        Ok(Self {
            struct_name: _struct.ident,
            user_table_name: None,
            user_schema_name: None,
            vis: _struct.vis,
            generics: _struct.generics,
            fields: parsed_fields,
            attrs: _struct.attrs,
        })
    }
}
