use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::query_operations::update::__details::*;
use crate::utils::macro_tokens::MacroTokens;

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
    let update_values_cloned = update_values.clone();

    if let Some(primary_key) = macro_data.get_primary_key_annotation() {
        let pk_ident = Ident::new(&primary_key, Span::call_site());

        update_ops_tokens.extend(quote! {
            /// Updates a database record that matches
            /// the current instance of a T type, returning a result
            /// indicating a possible failure querying the database.
            async fn update(&self) -> Result<u64, Box<dyn std::error::Error + Sync + std::marker::Send>> {
                let stmt = format!(
                    "UPDATE {} SET {} WHERE {} = ${:?}",
                    #table_schema_data, #str_columns_values, #primary_key, &self.#pk_ident
                );
                let update_values: &[&dyn canyon_sql::core::QueryParameter<'_>] = &[#(#update_values),*];

                <#ty as canyon_sql::core::Transaction>::execute(stmt, update_values, "").await
            }
            /// Updates a database record that matches
            /// the current instance of a T type, returning a result
            /// indicating a possible failure querying the database with the
            /// specified datasource
            async fn update_with<'a, I>(&self, input: I)
                -> Result<u64, Box<dyn std::error::Error + Sync + std::marker::Send + 'a>>
            where I: canyon_sql::core::DbConnection + Send + 'a
            {
                let stmt = format!(
                    "UPDATE {} SET {} WHERE {} = ${:?}",
                    #table_schema_data, #str_columns_values, #primary_key, &self.#pk_ident
                );
                let update_values: &[&dyn canyon_sql::core::QueryParameter<'_>] = &[#(#update_values_cloned),*];

                <#ty as canyon_sql::core::Transaction>::execute(stmt, update_values, input).await
            }
        });
    } else {
        // If there's no primary key, update method over self won't be available.
        // Use instead the update associated function of the querybuilder
        let update_err_tokens = create_update_err_macro(ty);
        let update_err_with_tokens = create_update_err_with_macro(ty);

        update_ops_tokens.extend(quote! {
            #update_err_tokens
            #update_err_with_tokens
        });
    }

    // let querybuilder_update_tokens = generate_update_query_tokens(ty, table_schema_data);
    // update_ops_tokens.extend(querybuilder_update_tokens);

    update_ops_tokens
}

/// Generates the TokenStream for the __update() CRUD operation
/// being the query generated with the [`QueryBuilder`]
fn generate_update_query_tokens(ty: &Ident, table_schema_data: &String) -> TokenStream {
    quote! {
        /// Generates a [`canyon_sql::query::UpdateQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `UPDATE table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        fn update_query<'a>() -> canyon_sql::query::UpdateQueryBuilder<'a, #ty, &'a str> {
            canyon_sql::query::UpdateQueryBuilder::new(#table_schema_data, "")
        }

        /// Generates a [`canyon_sql::query::UpdateQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `UPDATE table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        ///
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the input parameter
        fn update_query_with<'a, I>(input: I) -> canyon_sql::query::UpdateQueryBuilder<'a, #ty, I>
            where I: canyon_sql::core::DbConnection + Send + 'a
        {
            canyon_sql::query::UpdateQueryBuilder::new(#table_schema_data, input)
        }
    }
}

mod __details {
    use crate::query_operations::doc_comments;
    use crate::query_operations::macro_template::MacroOperationBuilder;
    use proc_macro2::{Ident, Span};

    pub fn create_update_err_macro(ty: &syn::Ident) -> MacroOperationBuilder {
        MacroOperationBuilder::new()
            .fn_name("update")
            .with_self_as_ref()
            .user_type(ty)
            .return_type(&Ident::new("u64", Span::call_site()))
            .raw_return()
            .add_doc_comment(doc_comments::UNAVAILABLE_CRUD_OP_ON_INSTANCE)
            .with_direct_error_return(doc_comments::UNAVAILABLE_CRUD_OP_ON_INSTANCE)
    }

    pub fn create_update_err_with_macro(ty: &syn::Ident) -> MacroOperationBuilder {
        MacroOperationBuilder::new()
            .fn_name("update_with")
            .with_self_as_ref()
            .with_input_param()
            .user_type(ty)
            .return_type(&Ident::new("u64", Span::call_site()))
            .raw_return()
            .add_doc_comment(doc_comments::UNAVAILABLE_CRUD_OP_ON_INSTANCE)
            .with_direct_error_return(doc_comments::UNAVAILABLE_CRUD_OP_ON_INSTANCE)
    }
}

#[cfg(test)]
mod update_tokens_tests {
    use crate::query_operations::consts::{
        INPUT_PARAM, LT_CONSTRAINT, RES_VOID_RET_TY, RES_VOID_RET_TY_LT, USER_MOCK_TY,
    };
    use crate::query_operations::update::__details::{
        create_update_err_macro, create_update_err_with_macro,
    };

    #[test]
    fn test_macro_builder_update_err() {
        let update_err_builder = create_update_err_macro(
            &USER_MOCK_TY.with(|user_mock_ty| user_mock_ty.borrow().clone()),
        );
        let update_err = update_err_builder.generate_tokens().to_string();

        assert!(update_err.contains("async fn update"));
        assert!(update_err.contains(RES_VOID_RET_TY));
    }

    #[test]
    fn test_macro_builder_update_err_with() {
        let update_err_with_builder = create_update_err_with_macro(
            &USER_MOCK_TY.with(|user_mock_ty| user_mock_ty.borrow().clone()),
        );
        let update_err_with = update_err_with_builder.generate_tokens().to_string();

        assert!(update_err_with.contains("async fn update_with"));
        assert!(update_err_with.contains(RES_VOID_RET_TY_LT));
        assert!(update_err_with.contains(LT_CONSTRAINT));
        assert!(update_err_with.contains(INPUT_PARAM));
    }
}
