use crate::query_operations::consts;
use crate::utils::helpers;
use crate::utils::macro_tokens::MacroTokens;
use crate::utils::primary_key_attribute::PrimaryKeyIndex;
use proc_macro2::TokenStream;
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
    let update_signature = quote! {
        async fn update(&self) -> Result<u64, Box<dyn std::error::Error + Sync + std::marker::Send>>
    };
    let update_with_signature = quote! {
        async fn update_with<'a, I>(&self, input: I)
            -> Result<u64, Box<dyn std::error::Error + Sync + std::marker::Send + 'a>>
        where I: canyon_sql::connection::DbConnection + Send + 'a
    };

    let mut update_ops_tokens = TokenStream::new();

    if let Some(primary_key) = macro_data.get_primary_key_field_annotation() {
        let ty = macro_data.ty;
        let (_, ty_generics, _) = macro_data.generics.split_for_impl();

        let update_columns = helpers::get_struct_fields_as_table_column_pairs_pk_parsed(macro_data);
        let pk = primary_key.ident;

        let pk_name = &primary_key.name;

        let update_values = macro_data
            .get_fields_idents_pk_parsed()
            .map(|ident| {
                quote! {
                    &self.#ident as &dyn canyon_sql::query::QueryParameter
                }
            })
            .chain(std::iter::once(quote! {
                &self.#pk as &dyn canyon_sql::query::QueryParameter
            }))
            .collect::<Vec<_>>();

        let query = quote! {
                canyon_sql::query::querybuilder::UpdateQueryBuilder::new_for(
                    #table_schema_data, // TODO: construct a const value
                    db_type,
                )
                    .set(vec![#(#update_columns),*])?
                    .r#where(
                        #pk_name,
                        canyon_sql::query::operators::Operator::Eq,
                    )
                    .build()?;
        };

        update_ops_tokens.extend(quote! {
            #update_signature {
                use canyon_sql::query::querybuilder::{QueryBuilderOps, UpdateQueryBuilderOps};

                let default_conn = canyon_sql::core::Canyon::instance()?.get_default_connection()?;
                let db_type = default_conn.get_database_type()?;

                let query = #query;
                let update_values: &[&dyn canyon_sql::query::QueryParameter] = &[#(#update_values),*];
                <#ty #ty_generics as canyon_sql::core::Transaction>::execute(query.as_ref(), update_values, default_conn).await
            }
            #update_with_signature {
                use canyon_sql::query::querybuilder::{QueryBuilderOps, UpdateQueryBuilderOps};
                let db_type = input.get_database_type()?;
                let query = #query;
                let update_values: &[&dyn canyon_sql::query::QueryParameter] = &[#(#update_values),*];
                input.execute(query.as_ref(), update_values).await
            }
        });
    } else {
        // If there's no primary key, update method over self won't be available.
        // Use instead the update associated function of the querybuilder
        let no_pk_err = __err::invalid_method_call_without_pk_present();
        update_ops_tokens.extend(quote! {
            #update_signature { #no_pk_err }
            #update_with_signature{ #no_pk_err }
        });
    }

    update_ops_tokens
}

fn generate_update_entity_tokens(table_schema_data: &str) -> TokenStream {
    let update_entity_signature = quote! {
        async fn update_entity<'canyon_lt, 'err_lt, Entity>(entity: &'canyon_lt Entity)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
        where Entity: canyon_sql::core::RowMapper
            + canyon_sql::query::bounds::Inspectionable<'canyon_lt>
            + Sync
            + 'canyon_lt
    };

    let update_entity_with_signature = quote! {
        async fn update_entity_with<'canyon_lt, 'err_lt, Entity, Input>(entity: &'canyon_lt Entity, input: Input)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
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
        fn update_query<'a>() -> canyon_sql::query::querybuilder::UpdateQueryBuilder<'a> {
            canyon_sql::query::querybuilder::UpdateQueryBuilder::new(#table_schema_data)
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
        fn update_query_with<'a>(database_type: canyon_sql::connection::DatabaseType) ->
            canyon_sql::query::querybuilder::UpdateQueryBuilder<'a> {
            canyon_sql::query::querybuilder::UpdateQueryBuilder::new_for(#table_schema_data, database_type)
        }
    }
}

mod __details {
    use super::*;
    use crate::utils::primary_key_attribute::PrimaryKeyAttribute;

    pub(crate) fn generate_update_entity_body(table_schema_data: &str) -> TokenStream {
        let update_entity_core_logic = generate_update_entity_pk_body_logic(table_schema_data);
        let no_pk_err = __err::generate_no_pk_error();

        quote! {
            if let Some(primary_key) = entity.primary_key() {
                #update_entity_core_logic

                let default_db_conn = canyon_sql::core::Canyon::instance()?
                    .get_default_connection()?;
                let _ = default_db_conn.execute(&stmt, &update_values).await?;
                Ok(())
            } else {
                #no_pk_err
            }
        }
    }

    pub(crate) fn generate_update_entity_with_body(table_schema_data: &str) -> TokenStream {
        let update_entity_core_logic = generate_update_entity_pk_body_logic(table_schema_data);
        let no_pk_err = __err::generate_no_pk_error();

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
            let pk_actual_value = entity.primary_key_actual_value();
            let update_columns = entity.fields_names();
            let update_values_pk_parsed = entity.fields_actual_values();

            let mut vec_columns_values: Vec<String> = Vec::new();
            for (i, column_name) in update_columns.to_vec().iter().enumerate() {
                let column_equal_value = format!("{} = ${}", column_name, i + 2);
                vec_columns_values.push(column_equal_value)
            }
            let col_vals_placeholders = vec_columns_values.join(", ");

            // Efficiently build argument list: pk first, then values
            let mut update_values: Vec<&dyn canyon_sql::query::QueryParameter> =
                Vec::with_capacity(1 + update_values_pk_parsed.len());
            update_values.push(pk_actual_value);
            update_values.extend(update_values_pk_parsed);

            let stmt = format!(
                "UPDATE {} SET {} WHERE {:?} = $1",
                #table_schema_data, col_vals_placeholders, primary_key
            );
        }
    }

    pub(crate) fn generate_update_stmt(
        table_schema_data: &str,
        macro_data: &MacroTokens,
        primary_key_attribute: &PrimaryKeyAttribute,
    ) -> TokenStream {
        let fields = macro_data.get_fields_idents_pk_parsed();

        let update_columns_and_values = fields.map(|ident| {
            let column_name = ident.to_string();
            quote! { (#column_name, &self.#ident) }
        });

        quote! {
            let stmt =
                canyon_sql::query::querybuilder::UpdateQueryBuilder::new_for(
                    #table_schema_data,
                    db_conn.get_database_type()?,
                )
                    .set(&[#(#update_columns_and_values),*])?
                    .r#where(
                        primary_key_column,
                        Operator::Eq,
                    )
                    .build()?;
        }
    }
}

mod __err {
    use crate::query_operations::consts;
    use proc_macro2::TokenStream;
    use quote::quote;

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

    pub(crate) fn invalid_method_call_without_pk_present() -> TokenStream {
        let err_msg = consts::UNAVAILABLE_CRUD_OP_ON_INSTANCE;
        quote! {
            Err(
                std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    #err_msg
                ).into_inner().unwrap()
            )
        } // TODO: waiting for creating our custom error types
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
