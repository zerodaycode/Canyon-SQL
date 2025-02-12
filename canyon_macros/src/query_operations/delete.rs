use crate::query_operations::delete::__details::{
    create_delete_err_macro, create_delete_err_with_macro, create_delete_macro,
    create_delete_with_macro,
};
use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Type;

/// Generates the TokenStream for the __delete() CRUD operation
/// returning a result, indicating a possible failure querying the database
pub fn generate_delete_tokens(macro_data: &MacroTokens, table_schema_data: &String) -> TokenStream {
    let mut delete_ops_tokens = TokenStream::new();

    let ty = macro_data.ty;
    let fields = macro_data.get_struct_fields();
    let pk = macro_data.get_primary_key_annotation();

    let ret_ty: Type = syn::parse_str("()").expect("Failed to parse unit type");
    let q_ret_ty: TokenStream = quote! {#ret_ty};

    if let Some(primary_key) = pk {
        let pk_field = fields
            .iter()
            .find(|f| *f.to_string() == primary_key)
            .expect(
                "Something really bad happened finding the Ident for the pk field on the delete",
            );
        let pk_field_value =
            quote! { &self.#pk_field as &dyn canyon_sql::core::QueryParameter<'_> };

        let stmt = format!(
            "DELETE FROM {} WHERE {:?} = $1",
            table_schema_data, primary_key
        );

        let delete_tokens = create_delete_macro(ty, &stmt, &pk_field_value, &q_ret_ty);
        let delete_with_tokens = create_delete_with_macro(ty, &stmt, &pk_field_value, &q_ret_ty);

        delete_ops_tokens.extend(quote! {
            #delete_tokens
            #delete_with_tokens
        });
    } else {
        let delete_err_tokens = create_delete_err_macro(ty, &q_ret_ty);
        let delete_err_with_tokens = create_delete_err_with_macro(ty, &q_ret_ty);

        delete_ops_tokens.extend(quote! {
            #delete_err_tokens
            #delete_err_with_tokens
        });
    }

    let delete_with_querybuilder = generate_delete_query_tokens(ty, table_schema_data);
    delete_ops_tokens.extend(delete_with_querybuilder);
    
    delete_ops_tokens
}

/// Generates the TokenStream for the __delete() CRUD operation as a
/// [`query_elements::query_builder::QueryBuilder<'a, #ty>`]
fn generate_delete_query_tokens(ty: &Ident, table_schema_data: &str) -> TokenStream {
    quote! {
        /// Generates a [`canyon_sql::query::DeleteQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `DELETE FROM table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        fn delete_query<'a>() -> canyon_sql::query::DeleteQueryBuilder<'a, #ty, &'a str> {
            canyon_sql::query::DeleteQueryBuilder::new(#table_schema_data, "")
        }

        /// Generates a [`canyon_sql::query::DeleteQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `DELETE FROM table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        ///
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, selected with the input parameter
        fn delete_query_with<'a, I>(input: I) -> canyon_sql::query::DeleteQueryBuilder<'a, #ty, I>
            where I: canyon_sql::core::DbConnection + Send + 'a
        {
            canyon_sql::query::DeleteQueryBuilder::new(#table_schema_data, input)
        }
    }
}

// NOTE: The delete operations shouldn't be using TransactionMethod::QueryRows
// This should be refactored on the future
mod __details {

    use super::*;
    use crate::query_operations::doc_comments;
    use crate::query_operations::macro_template::{MacroOperationBuilder, TransactionMethod};

    pub fn create_delete_macro(
        ty: &Ident,
        stmt: &str,
        pk_field_value: &TokenStream,
        ret_ty: &TokenStream,
    ) -> MacroOperationBuilder {
        MacroOperationBuilder::new()
            .fn_name("delete")
            .with_self_as_ref()
            .user_type(ty)
            .return_type_ts(ret_ty)
            .raw_return()
            .add_doc_comment(doc_comments::DELETE)
            .query_string(stmt)
            .forwarded_parameters(quote! {&[#pk_field_value]})
            .propagate_transaction_result()
            .with_transaction_method(TransactionMethod::Execute)
            .raw_return()
            .with_no_result_value()
    }

    pub fn create_delete_with_macro(
        ty: &Ident,
        stmt: &str,
        pk_field_value: &TokenStream,
        ret_ty: &TokenStream,
    ) -> MacroOperationBuilder {
        MacroOperationBuilder::new()
            .fn_name("delete_with")
            .with_self_as_ref()
            .with_input_param()
            .user_type(ty)
            .return_type_ts(ret_ty)
            .raw_return()
            .add_doc_comment(doc_comments::DELETE)
            .add_doc_comment(doc_comments::DS_ADVERTISING)
            .query_string(stmt)
            .forwarded_parameters(quote! {&[#pk_field_value]})
            .propagate_transaction_result()
            .with_transaction_method(TransactionMethod::Execute)
            .raw_return()
            .with_no_result_value()
    }

    pub fn create_delete_err_macro(ty: &Ident, ret_ty: &TokenStream) -> MacroOperationBuilder {
        MacroOperationBuilder::new()
            .fn_name("delete")
            .with_self_as_ref()
            .user_type(ty)
            .return_type_ts(ret_ty)
            .raw_return()
            .add_doc_comment(doc_comments::UNAVAILABLE_CRUD_OP_ON_INSTANCE)
            .with_direct_error_return(doc_comments::UNAVAILABLE_CRUD_OP_ON_INSTANCE)
    }

    pub fn create_delete_err_with_macro(ty: &Ident, ret_ty: &TokenStream) -> MacroOperationBuilder {
        MacroOperationBuilder::new()
            .fn_name("delete_with")
            .with_self_as_ref()
            .with_input_param()
            .user_type(ty)
            .return_type_ts(ret_ty)
            .raw_return()
            .add_doc_comment(doc_comments::UNAVAILABLE_CRUD_OP_ON_INSTANCE)
            .with_direct_error_return(doc_comments::UNAVAILABLE_CRUD_OP_ON_INSTANCE)
    }
}

#[cfg(test)]
mod delete_tests {
    use super::__details::*;
    use crate::query_operations::consts::*;
    
    
    

    const DELETE_MOCK_STMT: &str = "DELETE FROM public.user WHERE user.id = 1";

    #[test]
    fn test_macro_builder_delete() {
        let delete_builder = create_delete_macro(
            &USER_MOCK_TY.with(|user_mock_ty| user_mock_ty.borrow().clone()),
            DELETE_MOCK_STMT,
            &PK_MOCK_FIELD_VALUE.with(|pk_field_mock_value| pk_field_mock_value.borrow().clone()),
            &VOID_RET_TY.with(|void_ret_ty| void_ret_ty.borrow().clone()),
        );
        let delete = delete_builder.generate_tokens().to_string();

        assert!(delete.contains("async fn delete"));
        assert!(delete.contains(RES_VOID_RET_TY));
    }

    #[test]
    fn test_macro_builder_delete_with() {
        let delete_builder = create_delete_with_macro(
            &USER_MOCK_TY.with(|user_mock_ty| user_mock_ty.borrow().clone()),
            DELETE_MOCK_STMT,
            &PK_MOCK_FIELD_VALUE.with(|pk_field_mock_value| pk_field_mock_value.borrow().clone()),
            &VOID_RET_TY.with(|void_ret_ty| void_ret_ty.borrow().clone()),
        );
        let delete_with = delete_builder.generate_tokens().to_string();

        assert!(delete_with.contains("async fn delete_with"));
        assert!(delete_with.contains(RES_VOID_RET_TY_LT));
        assert!(delete_with.contains(LT_CONSTRAINT));
        assert!(delete_with.contains(INPUT_PARAM));
    }

    #[test]
    fn test_macro_builder_delete_err() {
        let delete_err_builder = create_delete_err_macro(
            &USER_MOCK_TY.with(|user_mock_ty| user_mock_ty.borrow().clone()),
            &VOID_RET_TY.with(|void_ret_ty| void_ret_ty.borrow().clone()),
        );
        let delete_err = delete_err_builder.generate_tokens().to_string();

        assert!(delete_err.contains("async fn delete"));
        assert!(delete_err.contains(RES_VOID_RET_TY));
    }

    #[test]
    fn test_macro_builder_delete_err_with() {
        let delete_err_with_builder = create_delete_err_with_macro(
            &USER_MOCK_TY.with(|user_mock_ty| user_mock_ty.borrow().clone()),
            &VOID_RET_TY.with(|void_ret_ty| void_ret_ty.borrow().clone()),
        );
        let delete_err_with = delete_err_with_builder.generate_tokens().to_string();

        assert!(delete_err_with.contains("async fn delete_with"));
        assert!(delete_err_with.contains(RES_VOID_RET_TY_LT));
        assert!(delete_err_with.contains(LT_CONSTRAINT));
        assert!(delete_err_with.contains(INPUT_PARAM));
    }
}
