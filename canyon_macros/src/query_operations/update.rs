use crate::query_operations::consts;
use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

/// Generates the TokenStream for the __update() CRUD operation
pub fn generate_update_tokens(macro_data: &MacroTokens, table_schema_data: &String) -> TokenStream {
    let mut update_ops_tokens = TokenStream::new();

    let ty = macro_data.ty;
    let update_columns = macro_data.get_column_names_pk_parsed();
    let fields = macro_data.get_struct_fields();

    let mut vec_columns_values: Vec<String> = Vec::new();
    for (i, column_name) in update_columns.iter().enumerate() {
        let column_equal_value = format!("{} = ${}", column_name.to_owned(), i + 2);
        vec_columns_values.push(column_equal_value)
    }

    let str_columns_values = vec_columns_values.join(", ");

    let update_values = fields.iter().map(|ident| {
        quote! { &self.#ident }
    });

    let update_signature = quote! {
        /// Updates a database record that matches the current instance of a T type, returning a
        /// result indicating a possible failure querying the database.
        async fn update(&self) -> Result<u64, Box<dyn std::error::Error + Sync + std::marker::Send>>
    };
    let update_with_signature = quote! {
        async fn update_with<'a, I>(&self, input: I)
            -> Result<u64, Box<dyn std::error::Error + Sync + std::marker::Send + 'a>>
        where I: canyon_sql::core::DbConnection + Send + 'a
    };

    if let Some(primary_key) = macro_data.get_primary_key_annotation() {
        let pk_ident = Ident::new(&primary_key, Span::call_site());
        let stmt = quote! {&format!(
            "UPDATE {} SET {} WHERE {} = ${:?}",
            #table_schema_data, #str_columns_values, #primary_key, &self.#pk_ident
        )};
        let update_values = quote! {
            &[#(#update_values),*]
        };

        update_ops_tokens.extend(quote! {
            #update_signature {
                let update_values: &[&dyn canyon_sql::core::QueryParameter<'_>] = #update_values;
                <#ty as canyon_sql::core::Transaction>::execute(#stmt, update_values, "").await
            }
            #update_with_signature {
                let update_values: &[&dyn canyon_sql::core::QueryParameter<'_>] = #update_values;
                input.execute(#stmt, update_values).await
            }
        });
    } else {
        // If there's no primary key, update method over self won't be available.
        // Use instead the update associated function of the querybuilder
        let err_msg = consts::UNAVAILABLE_CRUD_OP_ON_INSTANCE;
        let no_pk_err = quote! {
            Err(
                std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    #err_msg
                ).into_inner().unwrap()
            )
        }; // TODO: waiting for creating our custom error types

        update_ops_tokens.extend(quote! {
            #update_signature { #no_pk_err }
            #update_with_signature{ #no_pk_err }
        });
    }

    let querybuilder_update_tokens = generate_update_querybuilder_tokens(table_schema_data);
    update_ops_tokens.extend(querybuilder_update_tokens);

    update_ops_tokens
}

/// Generates the TokenStream for the __update() CRUD operation
/// being the query generated with the [`QueryBuilder`]
fn generate_update_querybuilder_tokens(table_schema_data: &String) -> TokenStream {
    quote! {
        /// Generates a [`canyon_sql::query::querybuilder::UpdateQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `UPDATE table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        fn update_query<'a>() -> Result<
            canyon_sql::query::querybuilder::UpdateQueryBuilder<'a>,
            Box<(dyn std::error::Error + Send + Sync + 'a)>
        > {
            canyon_sql::query::querybuilder::UpdateQueryBuilder::new(#table_schema_data, canyon_sql::connection::DatabaseType::default_type()?)
        }

        /// Generates a [`canyon_sql::query::querybuilder::UpdateQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `UPDATE table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        ///
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the input parameter
        fn update_query_with<'a>(database_type: canyon_sql::connection::DatabaseType) -> Result<
            canyon_sql::query::querybuilder::UpdateQueryBuilder<'a>,
            Box<(dyn std::error::Error + Send + Sync + 'a)>
        > {
            canyon_sql::query::querybuilder::UpdateQueryBuilder::new(#table_schema_data, database_type)
        }
    }
}

//
// #[cfg(test)]
// mod update_tokens_tests {
//     use proc_macro2::Ident;
//     use crate::query_operations::consts;
//
//     // use crate::query_operations::consts::{
//     //     INPUT_PARAM, LT_CONSTRAINT, RES_VOID_RET_TY, RES_VOID_RET_TY_LT, USER_MOCK_TY,
//     // };
//     #[test]
//     fn test_create_update_macro() {
//         let ty = syn::parse_str::<Ident>("User").unwrap();
//         let mapper_ty = syn::parse_str::<Ident>("User").unwrap();
//         let tokens = crate::query_operations::read::__details::find_all_generators::create_find_all_macro(&ty, &mapper_ty, crate::query_operations::read::macro_builder_read_ops_tests::SELECT_ALL_STMT);
//         let generated = tokens.to_string();
//
//         assert!(generated.contains("async fn find_all"));
//         assert!(generated.contains(consts::RES_RET_TY));
//         assert!(generated.contains(crate::query_operations::read::macro_builder_read_ops_tests::SELECT_ALL_STMT));
//     }
//
//     #[test]
//     fn test_create_find_all_with_macro() {
//         let mapper_ty = syn::parse_str::<Ident>("User").unwrap();
//         let tokens = crate::query_operations::read::__details::find_all_generators::create_find_all_with_macro(&mapper_ty, crate::query_operations::read::macro_builder_read_ops_tests::SELECT_ALL_STMT);
//         let generated = tokens.to_string();
//
//         assert!(generated.contains("async fn find_all_with"));
//         assert!(generated.contains(consts::RES_RET_TY));
//         assert!(generated.contains(consts::LT_CONSTRAINT));
//         assert!(generated.contains(consts::INPUT_PARAM));
//         assert!(generated.contains(crate::query_operations::read::macro_builder_read_ops_tests::SELECT_ALL_STMT));
//     }
// }
