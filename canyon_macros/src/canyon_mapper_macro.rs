#![allow(unused_imports)]

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use regex::Regex;
use std::iter::Map;
use std::slice::Iter;
use syn::{DeriveInput, Type, Visibility};

#[cfg(feature = "mssql")]
const BY_VALUE_CONVERSION_TARGETS: [&str; 1] = ["String"];

pub fn canyon_mapper_impl_tokens(ast: MacroTokens) -> TokenStream {
    let mut row_mapper_tokens = TokenStream::new();

    let ty = ast.ty;
    let ty_str = ty.to_string();
    let fields = ast.fields();
    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let mut impl_methods = TokenStream::new();

    #[cfg(feature = "postgres")]
    let pg_implementation = create_postgres_fields_mapping(&ty_str, &fields);
    #[cfg(feature = "postgres")]
    impl_methods.extend(quote! {
        fn deserialize_postgresql(row: &canyon_sql::db_clients::tokio_postgres::Row) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Self {
                #(#pg_implementation),*
            })
        }
    });

    #[cfg(feature = "mssql")]
    let sqlserver_implementation = create_sqlserver_fields_mapping(&ty_str, &fields);
    #[cfg(feature = "mssql")]
    impl_methods.extend(quote! {
        fn deserialize_sqlserver(row: &canyon_sql::db_clients::tiberius::Row) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Self {
                #(#sqlserver_implementation),*
            })
        }
    });

    #[cfg(feature = "mysql")]
    let mysql_implementation = create_mysql_fields_mapping(&ty_str, &fields);
    #[cfg(feature = "mysql")]
    impl_methods.extend(quote! {
        fn deserialize_mysql(row: &canyon_sql::db_clients::mysql_async::Row) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Self {
                #(#mysql_implementation),*
            })
        }
    });

    row_mapper_tokens.extend(quote! {
        use crate::canyon_sql::crud::CrudOperations;
        impl #impl_generics canyon_sql::core::RowMapper for #ty #ty_generics #where_clause {
            type Output = #ty;
            #impl_methods
        }
    });

    let inspectionable_impl_tokens =
        __details::inspectionable_macro::generate_inspectionable_impl_tokens(&ast);
    row_mapper_tokens.extend(quote! {
        #inspectionable_impl_tokens
    });

    row_mapper_tokens
}

#[cfg(feature = "postgres")]
fn create_postgres_fields_mapping<'a>(
    ty: &'a str,
    fields: &'a [(Visibility, Ident, Type)],
) -> impl Iterator<Item = TokenStream> + use<'a> {
    fields.iter().map(|(_vis, ident, _ty)| {
        let ident_name = ident.to_string();
        let err = create_row_mapper_error_extracting_row(ident, ty, DatabaseType::PostgreSql);
        quote! {
            #ident: row.try_get::<&str, #_ty>(#ident_name).map_err(|_| #err)?
        }
    })
}

#[cfg(feature = "mysql")]
fn create_mysql_fields_mapping<'a>(
    ty: &'a str,
    fields: &'a [(Visibility, Ident, Type)],
) -> impl Iterator<Item = TokenStream> + use<'a> {
    fields.iter().map(|(_vis, ident, _ty)| {
        let ident_name = ident.to_string();
        let err = create_row_mapper_error_extracting_row(ident, ty, DatabaseType::MySQL);
        quote! {
            #ident: row.get_opt(#ident_name).ok_or_else(|| #err)??
        }
    })
}

#[cfg(feature = "mssql")]
fn create_sqlserver_fields_mapping<'a>(
    struct_ty: &'a str,
    fields: &'a [(Visibility, Ident, Type)],
) -> impl Iterator<Item = TokenStream> + use<'a> {
    fields.iter().map(move |(_vis, ident, ty)| {
        let ident_name = ident.to_string();
        let err = create_row_mapper_error_extracting_row(ident, struct_ty, DatabaseType::SqlServer);

        let target_field_type_str = get_field_type_as_string(ty);
        let field_deserialize_impl =
            handle_stupid_tiberius_sql_conversions(&target_field_type_str, &ident_name, err);

        quote! {
            #ident: #field_deserialize_impl
        }
    })
}

#[cfg(feature = "mssql")]
fn handle_stupid_tiberius_sql_conversions(
    target_type: &str,
    ident_name: &str,
    err: String,
) -> TokenStream {
    let is_opt_type = target_type.contains("Option");
    let handle_opt = if !is_opt_type {
        quote! { .ok_or_else(|| #err)? }
    } else {
        quote! {}
    };

    let deserializing_type = get_deserializing_type(target_type);
    let to_owned = if BY_VALUE_CONVERSION_TARGETS
        .iter()
        .any(|bv| target_type.contains(bv))
    {
        if is_opt_type {
            quote! { .map(|inner| inner.to_owned()) }
        } else {
            quote! { .to_owned() }
        }
    } else {
        quote! {}
    };

    quote! {
        // TODO: try_get
        row.get::<#deserializing_type, &str>(#ident_name)
            #handle_opt
            #to_owned
    }
}

#[cfg(feature = "mssql")]
fn get_deserializing_type(target_type: &str) -> TokenStream {
    let re = Regex::new(r"(?:Option\s*<\s*)?(?P<type>&?\w+)(?:\s*>)?").unwrap();
    re.captures(target_type)
        .map(|inner| String::from(&inner["type"]))
        .map(|tt| {
            if BY_VALUE_CONVERSION_TARGETS.contains(&tt.as_str()) {
                quote! { &str }
                // potentially others on demand on the future
            } else if tt.contains("Date") || tt.contains("Time") {
                let dt = Ident::new(tt.as_str(), Span::call_site());
                quote! { canyon_sql::date_time::#dt }
            } else {
                let tt = Ident::new(tt.as_str(), Span::call_site());
                quote! { #tt }
            }
        })
        .unwrap_or_else(|| {
            panic!(
                "Unable to process type: {} on the given struct for SqlServer",
                target_type
            )
        })
}

#[cfg(feature = "mssql")]
fn __get_deserializing_type_str(target_type: &str) -> String {
    let tt = get_deserializing_type(target_type);
    tt.to_string()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
}

use crate::utils::macro_tokens::MacroTokens;
use canyon_core::connection::database_type::DatabaseType;
#[cfg(feature = "mssql")]
use quote::ToTokens;

#[cfg(feature = "mssql")]
fn get_field_type_as_string(typ: &Type) -> String {
    match typ {
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

fn create_row_mapper_error_extracting_row(
    field_ident: &Ident,
    ty: &str,
    db_ty: DatabaseType,
) -> String {
    std::io::Error::other(format!(
        "Failed to retrieve the `{}` field for type: {} with {}",
        field_ident, ty, db_ty
    ))
    .to_string()
}

#[cfg(test)]
#[cfg(feature = "mssql")]
mod mapper_macro_tests {
    use crate::canyon_mapper_macro::__get_deserializing_type_str;

    #[test]
    fn test_regex_extraction_for_the_tiberius_target_types() {
        assert_eq!("&str", __get_deserializing_type_str("String"));
        assert_eq!("&str", __get_deserializing_type_str("Option<String>"));
        assert_eq!("i64", __get_deserializing_type_str("i64"));

        assert_eq!(
            "canyon_sql::date_time::DateTime",
            __get_deserializing_type_str("DateTime")
        );
        assert_eq!(
            "canyon_sql::date_time::NaiveDateTime",
            __get_deserializing_type_str("NaiveDateTime")
        );
    }
}

mod __details {
    use super::*;
    pub(crate) mod inspectionable_macro {
        use super::*;
        use crate::utils::helpers;
        use syn::{Field, Fields};

        pub(crate) fn generate_inspectionable_impl_tokens(ast: &MacroTokens) -> TokenStream {
            let ty = ast.ty;
            let ty_str = ty.to_string();
            let pk = ast.get_primary_key_field_annotation();
            let pk_ident_ts = pk.map(|pk| pk.ident);
            let pk_ty_ts = pk.map(|pk| pk.ty);

            let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

            let fields = ast.get_fields_idents_skipping_pk().collect::<Vec<_>>();
            let fields_values = get_fields_values_expr_tokens(&fields);
            let fields_names = get_fields_names_expr_tokens(&fields);

            let fields_as_column_refs = helpers::get_struct_fields_as_column_ref_token_stream(ast, true);
            let queries_placeholders = ast.placeholders_generator();

            let pk_opt_val = get_pk_ident_as_str(ast);
            let pk_actual_value = get_pk_actual_value_expr_tokens(ast);

            let set_pk_val_method = set_pk_val_method(&pk_ident_ts);
            let pk_assoc_ty = generate_pk_associated_type_tokens(&pk_ty_ts);

            quote! {
                impl #impl_generics canyon_sql::query::bounds::Inspectionable<'_> for #ty #ty_generics #where_clause {

                    type PrimaryKeyType = #pk_assoc_ty;

                    fn fields_actual_values(&self) -> Vec<&dyn canyon_sql::query::QueryParameter> {
                        vec![#(#fields_values),*]
                    }

                    fn fields_names(&self) -> &[&'static str] {
                        &[#(#fields_names),*]
                    }

                    fn fields_as_column_refs(&self) -> Vec<canyon_sql::query::ColumnRef<'static>> {
                        #fields_as_column_refs.collect()
                    }

                    fn queries_placeholders(&self) -> &'static str {
                        #queries_placeholders
                    }

                    fn primary_key(&self) -> Option<&'static str> {
                        #pk_opt_val
                    }

                    fn primary_key_st() -> Option<&'static str> {
                        #pk_opt_val
                    }

                    fn primary_key_actual_value(&self) -> &'_ (dyn canyon_sql::query::QueryParameter + '_) {
                         &#pk_actual_value
                    }

                    fn set_primary_key_actual_value(&mut self, value: Self::PrimaryKeyType) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                        #set_pk_val_method
                    }

                    fn primary_key_as_column_ref(&self) -> Option<canyon_sql::query::ColumnRef<'static>> {
                        self.primary_key()
                            .map(|pk| canyon_sql::query::ColumnRef::new(#ty_str, pk))
                    }
                }
            }
        }
    }

    fn get_fields_values_expr_tokens<'a>(
        fields: &'a Vec<&Ident>,
    ) -> Map<Iter<'a, &'a Ident>, fn(&'a &Ident) -> TokenStream> {
        fields.iter().map(|ident| {
            quote! { &self.#ident }
        })
    }

    fn get_fields_names_expr_tokens(fields: &Vec<&Ident>) -> Vec<String> {
        fields
            .iter()
            .map(|ident| ident.to_string())
            .collect::<Vec<_>>()
    }

    fn get_pk_ident_as_str(ast: &MacroTokens) -> TokenStream {
        match ast.get_primary_key_annotation() {
            Some(primary_key) => quote! { Some(#primary_key) },
            None => quote! { None },
        }
    }

    fn get_pk_actual_value_expr_tokens(ast: &MacroTokens) -> TokenStream {
        match ast.get_primary_key_annotation() {
            Some(primary_key) => {
                let pk_ident = Ident::new(&primary_key, Span::call_site());
                quote! { self.#pk_ident }
            }
            None => quote! { -1 }, // TODO: yeah, big todo :)
        }
    }

    fn generate_pk_associated_type_tokens(pk_ident_ts: &Option<&Type>) -> TokenStream {
        if let Some(pk_ty) = pk_ident_ts {
            quote! {
                #pk_ty
            }
        } else {
            quote! { i64 } // TODO: NoPrimaryKey
        }
    }

    fn set_pk_val_method(pk_ident_ts: &Option<&Ident>) -> TokenStream {
        if let Some(pk_ident) = pk_ident_ts {
            quote! {
                self.#pk_ident = value.into();
                Ok(())
            }
        } else {
            quote! {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "No primary key field defined for this entity"
                )) as Box<dyn std::error::Error + Send + Sync>)
            }
        }
    }
}
