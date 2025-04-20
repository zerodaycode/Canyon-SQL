use crate::utils::helpers::fields_with_types;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use regex::Regex;
use std::iter::Map;
use std::slice::Iter;
use syn::{DeriveInput, Type, Visibility};

#[cfg(feature = "mssql")]
const BY_VALUE_CONVERSION_TARGETS: [&str; 1] = ["String"];

pub fn canyon_mapper_impl_tokens(ast: DeriveInput) -> TokenStream {
    let ty = &ast.ident;
    let mut impl_methods = TokenStream::new();

    // Recovers the identifiers of the structs members
    let fields = fields_with_types(match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => {
            return syn::Error::new(ast.ident.span(), "CanyonMapper only works with Structs")
                .to_compile_error()
        }
    });

    #[cfg(feature = "postgres")]
    let pg_implementation = create_postgres_fields_mapping(&fields);
    #[cfg(feature = "postgres")]
    impl_methods.extend(quote! {
        fn deserialize_postgresql(row: &canyon_sql::db_clients::tokio_postgres::Row) -> Self::Output {
            Self {
                #(#pg_implementation),*
            }
        }
    });

    #[cfg(feature = "mssql")]
    let sqlserver_implementation = create_sqlserver_fields_mapping(&fields);
    #[cfg(feature = "mssql")]
    impl_methods.extend(quote! {
        fn deserialize_sqlserver(row: &canyon_sql::db_clients::tiberius::Row) -> Self::Output {
            Self {
                #(#sqlserver_implementation),*
            }
        }
    });

    #[cfg(feature = "mysql")]
    let mysql_implementation = create_mysql_fields_mapping(&fields);
    #[cfg(feature = "mysql")]
    impl_methods.extend(quote! {
        fn deserialize_mysql(row: &canyon_sql::db_clients::mysql_async::Row) -> Self::Output {
            Self {
                #(#mysql_implementation),*
            }
        }
    });

    quote! {
        impl canyon_sql::core::RowMapper for #ty {
            type Output = #ty;
            #impl_methods
        }
    }
}

#[cfg(feature = "postgres")]
#[allow(clippy::type_complexity)]
fn create_postgres_fields_mapping(
    fields: &[(Visibility, Ident, Type)],
) -> Map<Iter<'_, (Visibility, Ident, Type)>, fn(&'_ (Visibility, Ident, Type)) -> TokenStream> {
    fields.iter().map(|(_vis, ident, _ty)| {
        let ident_name = ident.to_string();
        quote! {
            #ident: row.try_get(#ident_name) // TODO: can we wrap RowMapper in a Result and propagate errors with ?\?
                .expect(format!("Failed to retrieve the {} field", #ident_name).as_ref())
        }
    })
}

#[cfg(feature = "mysql")]
#[allow(clippy::type_complexity)]fn create_mysql_fields_mapping(
    fields: &[(Visibility, Ident, Type)],
) -> Map<Iter<'_, (Visibility, Ident, Type)>, fn(&'_ (Visibility, Ident, Type)) -> TokenStream> {
    fields.iter().map(|(_vis, ident, _ty)| {
        let ident_name = ident.to_string();
        quote! {
            #ident: row.get(#ident_name)
                .expect(format!("Failed to retrieve the {} field", #ident_name).as_ref())
        }
    })
}

#[cfg(feature = "mssql")]
#[allow(clippy::type_complexity)]
fn create_sqlserver_fields_mapping(
    fields: &[(Visibility, Ident, Type)],
) -> Map<Iter<'_, (Visibility, Ident, Type)>, fn(&'_ (Visibility, Ident, Type)) -> TokenStream> {
    fields.iter().map(|(_vis, ident, ty)| {
        let ident_name = ident.to_string();

        let target_field_type_str = get_field_type_as_string(ty);
        let field_deserialize_impl =
            handle_stupid_tiberius_sql_conversions(&target_field_type_str, &ident_name);

        quote! {
            #ident: #field_deserialize_impl
        }
    })
}

#[cfg(feature = "mssql")]
fn handle_stupid_tiberius_sql_conversions(target_type: &str, ident_name: &str) -> TokenStream {
    let is_opt_type = target_type.contains("Option");
    let handle_opt = if !is_opt_type {
        quote! { .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref()) }
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
        .unwrap_or_else(|| panic!("Unable to process type: {} on the given struct for SqlServer",
            target_type))
}

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

#[cfg(test)]
#[cfg(feature = "mssql")]
mod mapper_macro_tests {
    use crate::canyon_mapper_macro::get_deserializing_type;

    #[test]
    fn test_regex_extraction_for_the_tiberius_target_types() {
        assert_eq!("&str", get_deserializing_type("String").to_string());
        assert_eq!("&str", get_deserializing_type("Option<String>").to_string());
        assert_eq!("i64", get_deserializing_type("i64").to_string());

        assert_eq!(
            "canyon_sql::date_time::DateTime",
            get_deserializing_type("DateTime").to_string()
        );
        assert_eq!(
            "canyon_sql::date_time::NaiveDateTime",
            get_deserializing_type("NaiveDateTime").to_string()
        );
    }
}
