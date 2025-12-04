use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

pub fn generate_delete_tokens(macro_data: &MacroTokens, table_schema_data: &str) -> TokenStream {
    let delete_method_ops = generate_delete_method_tokens(macro_data, table_schema_data);
    let delete_entity_ops = generate_delete_entity_tokens(table_schema_data);
    let delete_querybuilder_tokens = generate_delete_querybuilder_tokens(table_schema_data);

    quote! {
        #delete_method_ops
        #delete_entity_ops
        #delete_querybuilder_tokens
    }
}

/// Generates the TokenStream for the __delete() CRUD operation
/// returning a result, indicating a possible failure querying the database
pub fn generate_delete_method_tokens(
    macro_data: &MacroTokens,
    table_schema_data: &str,
) -> TokenStream {
    let mut delete_ops_tokens = TokenStream::new();

    let ty = macro_data.ty;
    let (_, ty_generics, _) = macro_data.generics.split_for_impl();
    let pk = macro_data.get_primary_key_annotation();

    let delete_signature = quote! {
        /// Deletes from a database entity the row that matches
        /// the current instance of a T type, returning a result
        /// indicating a possible failure querying the database.
        async fn delete(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    };
    let delete_with_signature = quote! {
        /// Deletes from a database entity the row that matches
        /// the current instance of a T type, returning a result
        /// indicating a possible failure querying the database with the specified datasource.
        async fn delete_with<'canyon, 'err, I>(&self, input: I) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'err)>>
            where I: canyon_sql::connection::DbConnection + Send + 'canyon
    };

    if let Some(primary_key) = pk {
        let pk_field = Ident::new(&primary_key, Span::call_site());
        let pk_field_value = quote! { &self.#pk_field as &dyn canyon_sql::query::QueryParameter };
        let delete_stmt = format!(
            "DELETE FROM {} WHERE {:?} = $1",
            table_schema_data, primary_key
        );

        delete_ops_tokens.extend(quote! {
            #delete_signature {
                <#ty #ty_generics as canyon_sql::core::Transaction>::execute(#delete_stmt, &[#pk_field_value], "").await?;
                Ok(())
            }

            #delete_with_signature {
                input.execute(#delete_stmt, &[#pk_field_value]).await?;
                Ok(())
            }
        });
    } else {
        // Delete operation over an instance isn't available without declaring a primary key.
        // The delete querybuilder variant must be used for the case when there's no pk declared
        let no_pk_error = quote! {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "You can't use the 'delete' method on a \
                CanyonEntity that does not have a #[primary_key] annotation. \
                If you need to perform an specific search, use the Querybuilder instead."
            ).into_inner().unwrap())
        };

        delete_ops_tokens.extend(quote! {
            #delete_signature { #no_pk_error }
            #delete_with_signature { #no_pk_error }
        });
    }

    delete_ops_tokens
}

pub fn generate_delete_entity_tokens(table_schema_data: &str) -> TokenStream {
    let delete_entity_signature = quote! {
        async fn delete_entity<'canyon, 'err, Entity>(entity: &'canyon Entity)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err>>
        where Entity: canyon_sql::core::RowMapper
            + canyon_sql::query::bounds::Inspectionable<'canyon>
            + Sync
            + 'canyon
    };

    let delete_entity_with_signature = quote! {
        async fn delete_entity_with<'canyon, 'err, Entity, Input>(entity: &'canyon Entity, input: Input)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err>>
        where
            Entity: canyon_sql::core::RowMapper
                + canyon_sql::query::bounds::Inspectionable<'canyon>
                + Sync
                + 'canyon,
            Input: canyon_sql::connection::DbConnection + Send + 'canyon
    };

    let delete_entity_body = __details::generate_delete_entity_body(table_schema_data);
    let delete_entity_with_body = __details::generate_delete_entity_with_body(&table_schema_data);

    quote! {
        #delete_entity_signature { #delete_entity_body }
        #delete_entity_with_signature { #delete_entity_with_body }
    }
}

/// Generates the TokenStream for the __delete() CRUD operation as a
/// [`query_elements::query_builder::QueryBuilder<'a, #ty>`]
fn generate_delete_querybuilder_tokens(table_schema_data: &str) -> TokenStream {
    quote! {
        /// Generates a [`canyon_sql::query::querybuilder::DeleteQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `DELETE FROM table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        fn delete_query<'canyon, 'err>() -> Result<
            canyon_sql::query::querybuilder::DeleteQueryBuilder<'canyon>,
            Box<(dyn std::error::Error + Send + Sync + 'err)>
        > where
    'canyon: 'err {
            canyon_sql::query::querybuilder::DeleteQueryBuilder::new(#table_schema_data)
        }

        /// Generates a [`canyon_sql::query::querybuilder::DeleteQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `DELETE FROM table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        ///
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, selected with the input parameter
        fn delete_query_with<'canyon, 'err>(database_type: canyon_sql::connection::DatabaseType)
        -> Result<
            canyon_sql::query::querybuilder::DeleteQueryBuilder<'canyon>,
            Box<(dyn std::error::Error + Send + Sync + 'err)>
        > where
    'canyon: 'err {
            canyon_sql::query::querybuilder::DeleteQueryBuilder::new_for(#table_schema_data, database_type)
        }
    }
}

mod __details {
    use super::*;
    use crate::query_operations::consts;

    pub(crate) fn generate_delete_entity_body(table_schema_data: &str) -> TokenStream {
        let delete_entity_core_logic = generate_delete_entity_pk_body_logic(table_schema_data);
        let no_pk_err = consts::generate_no_pk_error();

        quote! {
            if let Some(primary_key) = entity.primary_key() {
                #delete_entity_core_logic
                let default_db_conn = canyon_sql::core::Canyon::instance()?
                    .get_default_connection()?;
                let _ = default_db_conn.execute(&delete_stmt, &[pk_actual_value]).await?;
                Ok(())
            } else {
                #no_pk_err
            }
        }
    }

    pub(crate) fn generate_delete_entity_with_body(table_schema_data: &str) -> TokenStream {
        let delete_entity_core_logic = generate_delete_entity_pk_body_logic(table_schema_data);
        let no_pk_err = consts::generate_no_pk_error();

        quote! {
            if let Some(primary_key) = entity.primary_key() {
                #delete_entity_core_logic
                let _ = input.execute(&delete_stmt, &[pk_actual_value]).await?;
                Ok(())
            } else {
                #no_pk_err
            }
        }
    }

    fn generate_delete_entity_pk_body_logic(table_schema_data: &str) -> TokenStream {
        quote! {
            // let pk_actual_value = &entity.primary_key_actual_value() as &dyn canyon_sql::query::QueryParameter;
            let pk_actual_value = entity.primary_key_actual_value();
            let delete_stmt = format!(
                "DELETE FROM {} WHERE {:?} = $1",
                #table_schema_data, primary_key
            );
        }
    }
}

//
// // NOTE: The delete operations shouldn't be using TransactionMethod::QueryRows
// // This should be refactored on the future
// mod __details {
//
//     use super::*;
//     use crate::query_operations::doc_comments;
//     use crate::query_operations::macro_template::{MacroOperationBuilder, TransactionMethod};
//
//     pub fn create_delete_macro(
//         ty: &Ident,
//         stmt: &str,
//         pk_field_value: &TokenStream,
//         ret_ty: &TokenStream,
//     ) -> TokenStream {
//         MacroOperationBuilder::new()
//             .fn_name("delete")
//             .with_self_as_ref()
//             .user_type(ty)
//             .return_type_ts(ret_ty)
//             .raw_return()
//             .add_doc_comment(doc_comments::DELETE)
//             .query_string(stmt)
//             .forwarded_parameters(quote! {&[#pk_field_value]})
//             .propagate_transaction_result()
//             .with_transaction_method(TransactionMethod::Execute)
//             .raw_return()
//             .with_no_result_value()
//
//     }
//
//     pub fn create_delete_with_macro(
//         ty: &Ident,
//         stmt: &str,
//         pk_field_value: &TokenStream,
//         ret_ty: &TokenStream,
//     ) -> MacroOperationBuilder {
//         MacroOperationBuilder::new()
//             .fn_name("delete_with")
//             .with_self_as_ref()
//             .with_input_param()
//             .user_type(ty)
//             .return_type_ts(ret_ty)
//             .raw_return()
//             .add_doc_comment(doc_comments::DELETE)
//             .add_doc_comment(doc_comments::DS_ADVERTISING)
//             .query_string(stmt)
//             .forwarded_parameters(quote! {&[#pk_field_value]})
//             .propagate_transaction_result()
//             .with_transaction_method(TransactionMethod::Execute)
//             .raw_return()
//             .with_no_result_value()
//     }
//
//     pub fn create_delete_err_macro(ty: &Ident, ret_ty: &TokenStream) -> MacroOperationBuilder {
//         MacroOperationBuilder::new()
//             .fn_name("delete")
//             .with_self_as_ref()
//             .user_type(ty)
//             .return_type_ts(ret_ty)
//             .raw_return()
//             .add_doc_comment(doc_comments::UNAVAILABLE_CRUD_OP_ON_INSTANCE)
//             .with_direct_error_return(doc_comments::UNAVAILABLE_CRUD_OP_ON_INSTANCE)
//     }
//
//     pub fn create_delete_err_with_macro(ty: &Ident, ret_ty: &TokenStream) -> MacroOperationBuilder {
//         MacroOperationBuilder::new()
//             .fn_name("delete_with")
//             .with_self_as_ref()
//             .with_input_param()
//             .user_type(ty)
//             .return_type_ts(ret_ty)
//             .raw_return()
//             .add_doc_comment(doc_comments::UNAVAILABLE_CRUD_OP_ON_INSTANCE)
//             .with_direct_error_return(doc_comments::UNAVAILABLE_CRUD_OP_ON_INSTANCE)
//     }
// }
//
// #[cfg(test)]
// mod delete_tests {
//     use super::__details::*;
//     use crate::query_operations::consts::*;
//
//     const DELETE_MOCK_STMT: &str = "DELETE FROM public.user WHERE user.id = 1";
//
//     #[test]
//     fn test_macro_builder_delete() {
//         let delete_builder = create_delete_macro(
//             &USER_MOCK_TY.with(|user_mock_ty| user_mock_ty.borrow().clone()),
//             DELETE_MOCK_STMT,
//             &PK_MOCK_FIELD_VALUE.with(|pk_field_mock_value| pk_field_mock_value.borrow().clone()),
//             &VOID_RET_TY.with(|void_ret_ty| void_ret_ty.borrow().clone()),
//         );
//         let delete = delete_builder.generate_tokens().to_string();
//
//         assert!(delete.contains("async fn delete"));
//         assert!(delete.contains(RES_VOID_RET_TY));
//     }
//
//     #[test]
//     fn test_macro_builder_delete_with() {
//         let delete_builder = create_delete_with_macro(
//             &USER_MOCK_TY.with(|user_mock_ty| user_mock_ty.borrow().clone()),
//             DELETE_MOCK_STMT,
//             &PK_MOCK_FIELD_VALUE.with(|pk_field_mock_value| pk_field_mock_value.borrow().clone()),
//             &VOID_RET_TY.with(|void_ret_ty| void_ret_ty.borrow().clone()),
//         );
//         let delete_with = delete_builder.generate_tokens().to_string();
//
//         assert!(delete_with.contains("async fn delete_with"));
//         assert!(delete_with.contains(RES_VOID_RET_TY_LT));
//         assert!(delete_with.contains(LT_CONSTRAINT));
//         assert!(delete_with.contains(INPUT_PARAM));
//     }
//
//     #[test]
//     fn test_macro_builder_delete_err() {
//         let delete_err_builder = create_delete_err_macro(
//             &USER_MOCK_TY.with(|user_mock_ty| user_mock_ty.borrow().clone()),
//             &VOID_RET_TY.with(|void_ret_ty| void_ret_ty.borrow().clone()),
//         );
//         let delete_err = delete_err_builder.generate_tokens().to_string();
//
//         assert!(delete_err.contains("async fn delete"));
//         assert!(delete_err.contains(RES_VOID_RET_TY));
//     }
//
//     #[test]
//     fn test_macro_builder_delete_err_with() {
//         let delete_err_with_builder = create_delete_err_with_macro(
//             &USER_MOCK_TY.with(|user_mock_ty| user_mock_ty.borrow().clone()),
//             &VOID_RET_TY.with(|void_ret_ty| void_ret_ty.borrow().clone()),
//         );
//         let delete_err_with = delete_err_with_builder.generate_tokens().to_string();
//
//         assert!(delete_err_with.contains("async fn delete_with"));
//         assert!(delete_err_with.contains(RES_VOID_RET_TY_LT));
//         assert!(delete_err_with.contains(LT_CONSTRAINT));
//         assert!(delete_err_with.contains(INPUT_PARAM));
//     }
// }
