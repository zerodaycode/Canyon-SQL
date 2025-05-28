use crate::query_operations::consts;
use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

pub fn generate_update_tokens(macro_data: &MacroTokens, table_schema_data: &str) -> TokenStream {
    let update_method_ops = generate_update_method_tokens(macro_data, table_schema_data);
    let update_entity_ops = generate_update_entity_tokens(table_schema_data);
    let update_querybuilder_tokens = generate_update_querybuilder_tokens(table_schema_data);

    quote! {
        #update_method_ops
        #update_entity_ops
        #update_querybuilder_tokens
    }
}

fn generate_update_method_tokens(macro_data: &MacroTokens, table_schema_data: &str) -> TokenStream {
    let mut update_ops_tokens = TokenStream::new();

    let ty = macro_data.ty;
    let (_, ty_generics, _) = macro_data.generics.split_for_impl();
    let update_columns = macro_data.get_column_names_pk_parsed();
    let fields = macro_data.get_struct_fields();

    let mut vec_columns_values: Vec<String> = Vec::new();
    for (i, column_name) in update_columns.enumerate() {
        let column_equal_value = format!("{} = ${}", column_name, i + 2);
        vec_columns_values.push(column_equal_value)
    }

    let str_columns_values = vec_columns_values.join(", ");

    let update_values = fields.map(|ident| {
        quote! { &self.#ident }
    });

    let update_signature = quote! {
        async fn update(&self) -> Result<u64, Box<dyn std::error::Error + Sync + std::marker::Send>>
    };
    let update_with_signature = quote! {
        async fn update_with<'a, I>(&self, input: I)
            -> Result<u64, Box<dyn std::error::Error + Sync + std::marker::Send + 'a>>
        where I: canyon_sql::connection::DbConnection + Send + 'a
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
                let update_values: &[&dyn canyon_sql::query::QueryParameter<'_>] = #update_values;
                <#ty #ty_generics as canyon_sql::core::Transaction>::execute(#stmt, update_values, "").await
            }
            #update_with_signature {
                let update_values: &[&dyn canyon_sql::query::QueryParameter<'_>] = #update_values;
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

    update_ops_tokens
}

fn generate_update_entity_tokens(table_schema_data: &str) -> TokenStream {
    let update_entity_signature = quote! {
        async fn update_entity<'canyon_lt, Entity>(entity: &'canyon_lt Entity)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'canyon_lt>>
        where Entity: canyon_sql::core::RowMapper
            + canyon_sql::query::bounds::Inspectionable<'canyon_lt>
            + Sync
            + 'canyon_lt
    };

    let update_entity_with_signature = quote! {
        async fn update_entity_with<'canyon_lt, Entity, Input>(entity: &'canyon_lt Entity, input: Input)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'canyon_lt>>
        where
            Entity: canyon_sql::core::RowMapper
                + canyon_sql::query::bounds::Inspectionable<'canyon_lt>
                + Sync
                + 'canyon_lt,
            Input: canyon_sql::connection::DbConnection + Send + 'canyon_lt
    };

    let update_entity_body = __details::generate_update_entity_body(table_schema_data);
    let update_entity_with_body = __details::generate_update_entity_with_body(table_schema_data);

    quote! {
        #update_entity_signature { #update_entity_body }
        #update_entity_with_signature { #update_entity_with_body }
    }
}

/// Generates the TokenStream for the __update() CRUD operation
/// being the query generated with the [`QueryBuilder`]
fn generate_update_querybuilder_tokens(table_schema_data: &str) -> TokenStream {
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

mod __details {
    use super::*;

    pub(crate) fn generate_update_entity_body(table_schema_data: &str) -> TokenStream {
        let update_entity_core_logic = generate_update_entity_pk_body_logic(table_schema_data);
        let no_pk_err = generate_no_pk_error();

        quote! {
            if let Some(primary_key) = entity.primary_key() {
                #update_entity_core_logic

                let default_db_conn = canyon_sql::core::Canyon::instance()?
                    .get_default_connection()?;
                let _ = default_db_conn.lock().await.execute(&stmt, &update_values).await?;
                Ok(())
            } else {
                #no_pk_err
            }
        }
    }

    pub(crate) fn generate_update_entity_with_body(table_schema_data: &str) -> TokenStream {
        let update_entity_core_logic = generate_update_entity_pk_body_logic(table_schema_data);
        let no_pk_err = generate_no_pk_error();

        quote! {
            if let Some(primary_key) = entity.primary_key() {
                #update_entity_core_logic

                let _ = input.execute(&stmt, &update_values).await?;
                Ok(())
            } else {
                #no_pk_err
            }
        }
    }

    fn generate_update_entity_pk_body_logic(table_schema_data: &str) -> TokenStream {
        quote! {
            let pk_actual_value = &entity.primary_key_actual_value();
            let update_columns = entity.fields_names();
            let update_values = entity.fields_actual_values();

            let mut vec_columns_values: Vec<String> = Vec::new();
            for (i, column_name) in update_columns.to_vec().iter().enumerate() {
                let column_equal_value = format!("{} = ${}", column_name, i + 2);
                vec_columns_values.push(column_equal_value)
            }
            let str_columns_values = vec_columns_values.join(", ");

            let stmt = format!(
                "UPDATE {} SET {} WHERE {} = ${:?}",
                #table_schema_data, str_columns_values, primary_key, pk_actual_value
            );
        }
    }

    pub(crate) fn generate_no_pk_error() -> TokenStream {
        let err_msg = consts::UNAVAILABLE_CRUD_OP_ON_INSTANCE;
        quote! {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    #err_msg
                ).into_inner().unwrap()
            );
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
