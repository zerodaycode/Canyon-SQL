use crate::utils::canyon_crud_attribute::CanyonCrudAttribute;
use crate::utils::helpers;
use crate::utils::primary_key_attribute::PrimaryKeyAttribute;
use canyon_entities::field_annotation::EntityFieldAnnotation;
use canyon_entities::helpers::default_database_table_name_from_entity_name;
use proc_macro2::Ident;
use std::convert::TryFrom;
use syn::{Attribute, DeriveInput, Field, Fields, Generics, Type, Visibility};

/// Provides a convenient way of store the data for the TokenStream
/// received on a macro
#[allow(dead_code)]
pub struct MacroTokens<'a> {
    pub vis: &'a Visibility,
    pub ty: &'a Ident,
    pub generics: &'a Generics,
    pub attrs: &'a Vec<Attribute>,
    pub fields: &'a Fields,
    // -------- the new fields that must help to avoid recalculations every time that the user compiles
    pub(crate) canyon_crud_attribute: Option<CanyonCrudAttribute>, // Type level
    pub(crate) primary_key_attribute: Option<PrimaryKeyAttribute<'a>>, // Field level, quick access without iterations
}

impl<'a> MacroTokens<'a> {
    pub fn new(ast: &'a DeriveInput) -> Result<Self, syn::Error> {
        // TODO: impl syn::parse instead
        if let syn::Data::Struct(ref s) = ast.data {
            let attrs = &ast.attrs;

            let primary_key_attribute = __details::find_primary_key_field_annotation(&s.fields)
                .map(PrimaryKeyAttribute::from);

            let mut canyon_crud_attribute = None;
            for attr in attrs {
                if attr.path().is_ident("canyon_crud") {
                    canyon_crud_attribute = Some(attr.parse_args::<CanyonCrudAttribute>()?);
                }
            }

            Ok(Self {
                vis: &ast.vis,
                ty: &ast.ident,
                generics: &ast.generics,
                attrs: &ast.attrs,
                fields: &s.fields,
                canyon_crud_attribute,
                primary_key_attribute,
            })
        } else {
            __details::raise_canyon_crud_only_for_structs_err()
        }
    }

    pub fn retrieve_mapping_target_type(&self) -> &Option<Ident> {
        if let Some(canyon_crud_attribute) = &self.canyon_crud_attribute {
            &canyon_crud_attribute.maps_to
        } else {
            &None
        }
    }

    pub fn fields(&self) -> Vec<(Visibility, Ident, Type)> {
        self.fields
            .iter()
            .map(|field| {
                (
                    field.vis.clone(),
                    field.ident.clone().unwrap(),
                    field.clone().ty,
                )
            })
            .collect::<Vec<_>>()
    }

    /// Gives a Vec of tuples that contains the name and
    /// the type of every field on a Struct
    pub fn fields_with_types(&self) -> Vec<(&Ident, &Type)> {
        self.fields
            .iter()
            .map(|field| (field.ident.as_ref().unwrap(), &field.ty))
            .collect::<Vec<_>>()
    }

    pub fn get_struct_fields_as_table_column_pairs(&self) -> Vec<(String, String)> {
        let table_name = default_database_table_name_from_entity_name(&self.ty.to_string());

        self.fields
            .iter()
            .map(|field| {
                let column_name = field.ident.as_ref().unwrap().to_string();
                (table_name.clone(), column_name)
            })
            .collect()
    }

    pub fn get_columns_skipping_pk(&self) -> impl Iterator<Item = &Field> {
        let primary_key = self.primary_key_attribute.as_ref().map(|pk| &pk.ident);

        self.fields.iter().filter(move |field| {
            !matches!(
                (primary_key, field.ident.as_ref()),
                (Some(pk), Some(field_ident))
                    if field_ident == *pk && __details::primary_key_is_autoincremental(field)
            )
        })
    }

    pub fn get_struct_fields_as_table_column_pairs_skipping_pk(&self) -> Vec<(String, String)> {
        let table_name = default_database_table_name_from_entity_name(&self.ty.to_string());

        self.get_columns_skipping_pk()
            .map(|field| {
                let column_name = field
                    .ident
                    .as_ref()
                    .expect("Struct fields must be named")
                    .to_string();

                (table_name.clone(), column_name)
            })
            .collect()
    }

    /// Returns a collection with all the [`syn::Ident`] for all the type members, skipping (if present)
    /// the field which is annotated with #[primary_key]
    pub fn get_fields_idents_skipping_pk(&self) -> impl Iterator<Item = &Ident> {
        self.get_columns_skipping_pk()
            .map(|field| field.ident.as_ref().unwrap())
    }

    pub fn get_primary_key_field_annotation(&self) -> Option<&PrimaryKeyAttribute<'a>> {
        self.primary_key_attribute.as_ref()
    }

    /// Utility for find the primary key attribute (if exists) and the
    /// column name (field) which belongs
    pub fn get_primary_key_annotation(&self) -> Option<String> {
        self.get_primary_key_field_annotation()
            .map(|attr| attr.ident.clone().to_string())
    }

    pub fn get_primary_key_ident_and_type(&self) -> Option<(&Ident, &Type)> {
        let primary_key = self.get_primary_key_annotation();
        if let Some(primary_key) = primary_key {
            self.fields_with_types()
                .into_iter()
                .find(|(i, _t)| i.to_string() == primary_key)
        } else {
            None
        }
    }

    /// Utility for find the `foreign_key` attributes (if exists)
    pub fn get_fk_annotations(&self) -> Vec<(&Ident, EntityFieldAnnotation)> {
        let mut foreign_key_annotations = Vec::new();

        self.fields.iter().for_each(|field| {
            let attrs = field
                .attrs
                .iter()
                .filter(|attr| attr.path().segments[0].clone().ident == "foreign_key");
            attrs.for_each(|attr| {
                let fk_parse = EntityFieldAnnotation::try_from(&attr);
                if let Ok(fk_annotation) = fk_parse {
                    foreign_key_annotations.push((field.ident.as_ref().unwrap(), fk_annotation))
                }
            });
        });

        foreign_key_annotations
    }

    /// Boolean that returns true if the type contains a `#[primary_key]`
    /// annotation. False otherwise.
    pub fn type_has_primary_key(&self) -> bool {
        self.fields
            .iter()
            .any(|field| helpers::field_has_target_attribute(field, "primary_key"))
    }
}

mod __details {
    use crate::utils::helpers;
    use crate::utils::macro_tokens::MacroTokens;
    use crate::utils::primary_key_attribute::PrimaryKeyIndex;
    use canyon_entities::field_annotation::EntityFieldAnnotation;
    use proc_macro2::Span;
    use syn::{Field, Fields};

    pub(super) fn find_primary_key_field_annotation(
        fields: &Fields,
    ) -> Option<(PrimaryKeyIndex, &Field)> {
        fields.iter().enumerate().find_map(|index_and_field| {
            let idx = index_and_field.0;
            let field = index_and_field.1;
            if helpers::field_has_target_attribute(field, "primary_key") {
                Some((PrimaryKeyIndex(idx), field))
            } else {
                None
            }
        })
    }

    pub(super) fn primary_key_is_autoincremental(field: &Field) -> bool {
        field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("primary_key"))
            .and_then(|attr| EntityFieldAnnotation::try_from(&attr).ok())
            .is_none_or(|annotation| matches!(annotation, EntityFieldAnnotation::PrimaryKey(true)))
    }

    pub(crate) fn raise_canyon_crud_only_for_structs_err<'a>() -> Result<MacroTokens<'a>, syn::Error>
    {
        Err(syn::Error::new(
            Span::call_site(),
            "CanyonCrud may only be implemented for structs",
        ))
    }
}
