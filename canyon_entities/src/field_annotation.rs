use proc_macro2::Ident;
use quote::ToTokens;
use std::{convert::TryFrom, fmt};
use syn::{
    Attribute, Expr, Lit, MetaNameValue, Path, PathArguments, Token, punctuated::Punctuated,
};

/// The available annotations for a field that belongs to any struct
/// annotated with `#[canyon_entity]`.
#[derive(Clone)]
pub enum EntityFieldAnnotation {
    /// Primary-key metadata, including whether the value is generated automatically.
    PrimaryKey(bool),
    /// A typed reference to the related entity and one of its fields.
    ForeignKey {
        /// Rust entity type that owns the referenced field.
        entity: Path,
        /// Referenced Rust field. Its identifier is also the mapped database column.
        field: Ident,
    },
}

impl fmt::Debug for EntityFieldAnnotation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.get_as_string())
    }
}

impl EntityFieldAnnotation {
    /// Returns the data of the [`EntityFieldAnnotation`] in an understandable format for
    /// operations that require character matching.
    pub fn get_as_string(&self) -> String {
        match self {
            Self::PrimaryKey(autoincremental) => {
                format!("Annotation: PrimaryKey, Autoincremental: {autoincremental}")
            }
            Self::ForeignKey { entity, field } => {
                let entity = entity.to_token_stream().to_string().replace(' ', "");
                format!("Annotation: ForeignKey, Reference: {entity}::{field}")
            }
        }
    }

    fn parse_primary_key(
        ident: &Ident,
        args: syn::Result<Punctuated<MetaNameValue, Token![,]>>,
    ) -> syn::Result<Self> {
        let Ok(args) = args else {
            return Ok(Self::PrimaryKey(true));
        };

        let mut autoincremental = None;

        for arg in &args {
            match arg_key(arg)?.as_str() {
                "autoincremental" => {
                    autoincremental = Some(parse_bool_value(arg)?);
                }
                unknown => return Err(unknown_argument(arg, unknown)),
            }
        }

        autoincremental.map(Self::PrimaryKey).ok_or_else(|| {
            syn::Error::new_spanned(
                ident,
                "Missing `autoincremental` argument on the Primary Key annotation",
            )
        })
    }

    fn parse_foreign_key(
        ident: &Ident,
        args: syn::Result<Punctuated<MetaNameValue, Token![,]>>,
    ) -> syn::Result<Self> {
        let args = args.map_err(|error| {
            syn::Error::new_spanned(ident, format!("Error generating the Foreign Key: {error}"))
        })?;

        let mut reference = None;

        for arg in &args {
            match arg_key(arg)?.as_str() {
                "references" if reference.is_none() => {
                    reference = Some(parse_foreign_key_reference(arg)?);
                }
                "references" => {
                    return Err(syn::Error::new_spanned(
                        &arg.path,
                        "Duplicate `references` argument on the Foreign Key annotation",
                    ));
                }
                unknown => return Err(unknown_argument(arg, unknown)),
            }
        }

        let (entity, field) = reference.ok_or_else(|| {
            syn::Error::new_spanned(
                ident,
                "Missing `references` argument on the Foreign Key annotation",
            )
        })?;

        Ok(Self::ForeignKey { entity, field })
    }
}

impl TryFrom<&&Attribute> for EntityFieldAnnotation {
    type Error = syn::Error;

    fn try_from(attribute: &&Attribute) -> Result<Self, Self::Error> {
        let ident = attribute
            .path()
            .get_ident()
            .ok_or_else(|| syn::Error::new_spanned(attribute.path(), "Expected attribute ident"))?;

        let args =
            attribute.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated);

        match ident.to_string().as_str() {
            "primary_key" => Self::parse_primary_key(ident, args),
            "foreign_key" => Self::parse_foreign_key(ident, args),
            _ => Err(syn::Error::new_spanned(
                ident,
                format!("Unknown attribute `{ident}`"),
            )),
        }
    }
}

fn arg_key(arg: &MetaNameValue) -> syn::Result<String> {
    arg.path
        .get_ident()
        .map(ToString::to_string)
        .ok_or_else(|| syn::Error::new_spanned(&arg.path, "Expected argument ident"))
}

fn parse_string_value(arg: &MetaNameValue) -> syn::Result<String> {
    match &arg.value {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(lit) => Ok(lit.value()),
            _ => Err(syn::Error::new_spanned(
                &arg.value,
                "Expected string literal",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            &arg.value,
            "Expected literal expression",
        )),
    }
}

fn parse_foreign_key_reference(arg: &MetaNameValue) -> syn::Result<(Path, Ident)> {
    let Expr::Path(reference) = &arg.value else {
        return Err(syn::Error::new_spanned(
            &arg.value,
            "Expected a typed foreign-key reference such as `Team::id`",
        ));
    };

    if reference.qself.is_some() {
        return Err(syn::Error::new_spanned(
            &arg.value,
            "Qualified self types are not supported in foreign-key references",
        ));
    }

    let mut entity = reference.path.clone();
    let field = entity
        .segments
        .pop()
        .map(|pair| pair.into_value())
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &arg.value,
                "A foreign-key reference must contain an entity and a field",
            )
        })?;

    if entity.segments.trailing_punct() {
        entity.segments.pop_punct();
    }

    if entity.segments.is_empty() || !matches!(field.arguments, PathArguments::None) {
        return Err(syn::Error::new_spanned(
            &arg.value,
            "A foreign-key reference must have the form `Entity::field`",
        ));
    }

    Ok((entity, field.ident))
}

fn parse_bool_value(arg: &MetaNameValue) -> syn::Result<bool> {
    parse_string_value(arg).and_then(|value| {
        value.parse::<bool>().map_err(|_| {
            syn::Error::new_spanned(
                &arg.value,
                format!("Expected boolean string literal, found `{value}`"),
            )
        })
    })
}

fn unknown_argument(arg: &MetaNameValue, ident: &str) -> syn::Error {
    syn::Error::new_spanned(&arg.path, format!("Unknown annotation argument `{ident}`"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Attribute, Field, parse_quote};

    fn annotation_from(attribute: &Attribute) -> syn::Result<EntityFieldAnnotation> {
        EntityFieldAnnotation::try_from(&attribute)
    }

    fn field_attribute(field: &Field) -> &Attribute {
        field
            .attrs
            .first()
            .expect("test field must have one attribute")
    }

    #[test]
    fn parses_primary_key_without_arguments_as_autoincremental() {
        let field: Field = parse_quote! {
            #[primary_key]
            id: i32
        };

        let annotation = annotation_from(field_attribute(&field)).unwrap();

        assert!(matches!(
            annotation,
            EntityFieldAnnotation::PrimaryKey(true)
        ));
    }

    #[test]
    fn parses_primary_key_with_autoincremental_enabled() {
        let field: Field = parse_quote! {
            #[primary_key(autoincremental = "true")]
            id: i32
        };

        let annotation = annotation_from(field_attribute(&field)).unwrap();

        assert!(matches!(
            annotation,
            EntityFieldAnnotation::PrimaryKey(true)
        ));
    }

    #[test]
    fn parses_primary_key_with_autoincremental_disabled() {
        let field: Field = parse_quote! {
            #[primary_key(autoincremental = "false")]
            id: i32
        };

        let annotation = annotation_from(field_attribute(&field)).unwrap();

        assert!(matches!(
            annotation,
            EntityFieldAnnotation::PrimaryKey(false)
        ));
    }

    #[test]
    fn rejects_primary_key_with_unknown_argument() {
        let field: Field = parse_quote! {
            #[primary_key(foo = "true")]
            id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Unknown annotation argument `foo`")
        );
    }

    #[test]
    fn rejects_primary_key_with_non_boolean_value() {
        let field: Field = parse_quote! {
            #[primary_key(autoincremental = "yes")]
            id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Expected boolean string literal, found `yes`")
        );
    }

    #[test]
    fn rejects_primary_key_with_non_string_literal_value() {
        let field: Field = parse_quote! {
            #[primary_key(autoincremental = true)]
            id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(error.to_string().contains("Expected string literal"));
    }

    #[test]
    fn parses_foreign_key() {
        let field: Field = parse_quote! {
            #[foreign_key(references = User::id)]
            user_id: i32
        };

        let annotation = annotation_from(field_attribute(&field)).unwrap();

        match annotation {
            EntityFieldAnnotation::ForeignKey { entity, field } => {
                assert_eq!(entity.to_token_stream().to_string(), "User");
                assert_eq!(field, "id");
            }
            EntityFieldAnnotation::PrimaryKey(_) => panic!("expected foreign key annotation"),
        }
    }

    #[test]
    fn parses_qualified_foreign_key_reference() {
        let field: Field = parse_quote! {
            #[foreign_key(references = crate::models::User::external_id)]
            user_id: i32
        };

        let annotation = annotation_from(field_attribute(&field)).unwrap();

        match annotation {
            EntityFieldAnnotation::ForeignKey { entity, field } => {
                assert_eq!(
                    entity.to_token_stream().to_string(),
                    "crate :: models :: User"
                );
                assert_eq!(field, "external_id");
            }
            EntityFieldAnnotation::PrimaryKey(_) => panic!("expected foreign key annotation"),
        }
    }

    #[test]
    fn rejects_foreign_key_without_arguments() {
        let field: Field = parse_quote! {
            #[foreign_key]
            user_id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Error generating the Foreign Key")
        );
    }

    #[test]
    fn rejects_foreign_key_without_references_argument() {
        let field: Field = parse_quote! {
            #[foreign_key(cascade = true)]
            user_id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Unknown annotation argument `cascade`")
        );
    }

    #[test]
    fn rejects_foreign_key_reference_without_a_field() {
        let field: Field = parse_quote! {
            #[foreign_key(references = User)]
            user_id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("A foreign-key reference must have the form `Entity::field`")
        );
    }

    #[test]
    fn rejects_duplicate_foreign_key_references() {
        let field: Field = parse_quote! {
            #[foreign_key(references = User::id, references = Account::id)]
            user_id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Duplicate `references` argument")
        );
    }

    #[test]
    fn rejects_string_foreign_key_references() {
        let field: Field = parse_quote! {
            #[foreign_key(references = "User::id")]
            user_id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Expected a typed foreign-key reference such as `Team::id`")
        );
    }

    #[test]
    fn rejects_the_old_foreign_key_shape() {
        let field: Field = parse_quote! {
            #[foreign_key(entity = "User", table = "users", column = "id")]
            user_id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Unknown annotation argument `entity`")
        );
    }

    #[test]
    fn rejects_unknown_attribute() {
        let field: Field = parse_quote! {
            #[indexed]
            id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(error.to_string().contains("Unknown attribute `indexed`"));
    }

    #[test]
    fn formats_primary_key_annotation_as_string() {
        let annotation = EntityFieldAnnotation::PrimaryKey(true);

        assert_eq!(
            annotation.get_as_string(),
            "Annotation: PrimaryKey, Autoincremental: true"
        );
    }

    #[test]
    fn formats_foreign_key_annotation_as_string() {
        let annotation = EntityFieldAnnotation::ForeignKey {
            entity: parse_quote!(crate::models::User),
            field: parse_quote!(id),
        };

        assert_eq!(
            annotation.get_as_string(),
            "Annotation: ForeignKey, Reference: crate::models::User::id"
        );
    }
}
