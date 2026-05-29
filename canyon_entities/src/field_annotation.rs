use proc_macro2::Ident;
use std::convert::TryFrom;
use syn::{Attribute, Expr, Lit, MetaNameValue, Token, punctuated::Punctuated};

/// The available annotations for a field that belongs to any struct
/// annotated with `#[canyon_entity]`.
#[derive(Debug, Clone)]
pub enum EntityFieldAnnotation {
    PrimaryKey(bool),
    ForeignKey(String, String),
}

impl EntityFieldAnnotation {
    /// Returns the data of the [`EntityFieldAnnotation`] in an understandable format for
    /// operations that require character matching.
    pub fn get_as_string(&self) -> String {
        match self {
            Self::PrimaryKey(autoincremental) => {
                format!("Annotation: PrimaryKey, Autoincremental: {autoincremental}")
            }
            Self::ForeignKey(table, column) => {
                format!("Annotation: ForeignKey, Table: {table}, Column: {column}")
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

        let mut table = None;
        let mut column = None;

        for arg in &args {
            match arg_key(arg)?.as_str() {
                "table" => table = Some(parse_string_value(arg)?),
                "column" => column = Some(parse_string_value(arg)?),
                unknown => return Err(unknown_argument(arg, unknown)),
            }
        }

        Ok(Self::ForeignKey(
            table.ok_or_else(|| {
                syn::Error::new_spanned(
                    ident,
                    "Missing `table` argument on the Foreign Key annotation",
                )
            })?,
            column.ok_or_else(|| {
                syn::Error::new_spanned(
                    ident,
                    "Missing `column` argument on the Foreign Key annotation",
                )
            })?,
        ))
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
    fn rejects_primary_key_with_missing_autoincremental_argument_when_args_are_present() {
        let field: Field = parse_quote! {
            #[primary_key()]
            id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Missing `autoincremental` argument on the Primary Key annotation")
        );
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
            #[foreign_key(table = "users", column = "id")]
            user_id: i32
        };

        let annotation = annotation_from(field_attribute(&field)).unwrap();

        match annotation {
            EntityFieldAnnotation::ForeignKey(table, column) => {
                assert_eq!(table, "users");
                assert_eq!(column, "id");
            }
            EntityFieldAnnotation::PrimaryKey(_) => panic!("expected foreign key annotation"),
        }
    }

    #[test]
    fn parses_foreign_key_arguments_in_any_order() {
        let field: Field = parse_quote! {
            #[foreign_key(column = "id", table = "users")]
            user_id: i32
        };

        let annotation = annotation_from(field_attribute(&field)).unwrap();

        match annotation {
            EntityFieldAnnotation::ForeignKey(table, column) => {
                assert_eq!(table, "users");
                assert_eq!(column, "id");
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
    fn rejects_foreign_key_with_missing_table_argument() {
        let field: Field = parse_quote! {
            #[foreign_key(column = "id")]
            user_id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Missing `table` argument on the Foreign Key annotation")
        );
    }

    #[test]
    fn rejects_foreign_key_with_missing_column_argument() {
        let field: Field = parse_quote! {
            #[foreign_key(table = "users")]
            user_id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Missing `column` argument on the Foreign Key annotation")
        );
    }

    #[test]
    fn rejects_foreign_key_with_unknown_argument() {
        let field: Field = parse_quote! {
            #[foreign_key(table = "users", column = "id", cascade = "true")]
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
    fn rejects_foreign_key_with_non_string_table_value() {
        let field: Field = parse_quote! {
            #[foreign_key(table = users, column = "id")]
            user_id: i32
        };

        let error = annotation_from(field_attribute(&field)).unwrap_err();

        assert!(error.to_string().contains("Expected literal expression"));
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
        let annotation = EntityFieldAnnotation::ForeignKey("users".into(), "id".into());

        assert_eq!(
            annotation.get_as_string(),
            "Annotation: ForeignKey, Table: users, Column: id"
        );
    }
}
