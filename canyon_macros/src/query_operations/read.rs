use crate::utils::macro_tokens::MacroTokens;
use canyon_core::query::querybuilder::{SelectQueryBuilder, SelectQueryBuilderOps};
use canyon_core::query::querybuilder::syntax::table_metadata::TableMetadata;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};

/// Facade function that acts as the unique API for export to the real macro implementation
/// of all the generated macros for the READ operations
pub fn generate_read_operations_tokens(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &TableMetadata<'_>,
) -> TokenStream {
    let ty = macro_data.ty;
    let mapper_ty = macro_data
        .retrieve_mapping_target_type()
        .as_ref()
        .unwrap_or(ty);

    let cols = macro_data.get_column_names_pk_parsed().collect::<Vec<_>>();
    let find_all_query = SelectQueryBuilder::new(table_schema_data.clone())
        .expect("Unexpected error creating a SelectQueryBuilder for the find_all operations")
        .with_columns(&cols);

    match find_all_query.build() {
        Ok(query) => {
            let sql = query.as_ref();

            let find_all_tokens = generate_find_all_operations_tokens(mapper_ty, sql);
            let count_tokens = generate_count_operations_tokens(sql);
            let find_by_pk_tokens = generate_find_by_pk_operations_tokens(macro_data, sql);
            let read_querybuilder_ops = generate_select_querybuilder_tokens(sql);

            quote! {
                #find_all_tokens
                #read_querybuilder_ops
                #count_tokens
                #find_by_pk_tokens
            }
        },
        Err(e) => {
            syn::Error::new(mapper_ty.span(), format!("Failed to build query: {e}"))
                .to_compile_error()
        }
    }
}

fn generate_find_all_operations_tokens(
    mapper_ty: &Ident,
    find_all_query: &str
) -> TokenStream {
    let find_all = __details::find_all_generators::create_find_all_macro(mapper_ty, &find_all_query);
    let find_all_with =
        __details::find_all_generators::create_find_all_with_macro(mapper_ty, &find_all_query);

    quote! {
        #find_all
        #find_all_with
    }
}

fn generate_select_querybuilder_tokens(table_schema_data: &str) -> TokenStream {
    quote! {
        /// Generates a [`canyon_sql::query::querybuilder::SelectQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It generates a Query `SELECT * FROM  table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        fn select_query<'a>()
            -> Result<
                canyon_sql::query::querybuilder::SelectQueryBuilder<'a>,
                Box<dyn std::error::Error + Send + Sync + 'a>
            >
        {
            canyon_sql::query::querybuilder::SelectQueryBuilder::new(#table_schema_data)
        }

        /// Generates a [`canyon_sql::query::querybuilder::SelectQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It generates a Query `SELECT * FROM  table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        fn select_query_with<'a>(database_type: canyon_sql::connection::DatabaseType)
            -> Result<
                canyon_sql::query::querybuilder::SelectQueryBuilder<'a>,
                Box<dyn std::error::Error + Send + Sync + 'a>
        > {
            canyon_sql::query::querybuilder::SelectQueryBuilder::new_for(#table_schema_data, database_type)
        }
    }
}

fn generate_count_operations_tokens(table_schema_data: &str) -> TokenStream {
    let count_stmt = format!("SELECT COUNT(*) FROM {table_schema_data}");
    let count = __details::count_generators::create_count_macro(&count_stmt);
    let count_with = __details::count_generators::create_count_with_macro(&count_stmt);

    quote! {
        #count
        #count_with
    }
}

fn generate_find_by_pk_operations_tokens(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &str
) -> TokenStream {
    let ty = macro_data.ty;
    let mapper_ty = macro_data.retrieve_mapping_target_type().as_ref();
    let pk = macro_data.get_primary_key_annotation();

    let base_body = if let Some(compile_time_known_pk) = pk {
        Some(quote! {
            let stmt = format!(
                "SELECT * FROM {} WHERE {} = $1",
                #table_schema_data, #compile_time_known_pk
            );
        })
    } else {
        let tt = mapper_ty.unwrap_or(ty).to_token_stream();
        Some(quote! {
            use canyon_sql::query::bounds::Inspectionable;
            let pk = <#tt as Inspectionable>::primary_key_st()
                .ok_or_else(|| "No primary key found for this instance")?;
            let stmt = format!(
                "SELECT * FROM {} WHERE {} = $1",
                #table_schema_data,
                pk
            );
        })
    };

    let mapper_ty = mapper_ty.unwrap_or(ty);
    let find_by_pk =
        __details::find_by_pk_generators::create_find_by_pk_macro(mapper_ty, &base_body);
    let find_by_pk_with =
        __details::find_by_pk_generators::create_find_by_pk_with(mapper_ty, &base_body);

    quote! {
        #find_by_pk
        #find_by_pk_with
    }
}

mod __details {
    use quote::quote;
    use syn::Ident;

    pub mod find_all_generators {
        use super::*;
        use proc_macro2::TokenStream;

        pub fn create_find_all_macro(mapper_ty: &Ident, stmt: &str) -> TokenStream {
            quote! {
                async fn find_all()
                    -> Result<Vec<#mapper_ty>, Box<(dyn std::error::Error + Send + Sync)>>
                {
                    let default_db_conn = canyon_sql::core::Canyon::instance()?.get_default_connection()?;
                    default_db_conn.query(#stmt, &[]).await
                }
            }
        }

        pub fn create_find_all_with_macro(mapper_ty: &Ident, stmt: &str) -> TokenStream {
            quote! {
                async fn find_all_with<'a, I>(input: I)
                    -> Result<Vec<#mapper_ty>, Box<(dyn std::error::Error + Send + Sync)>>
                where
                    I: canyon_sql::connection::DbConnection + Send + 'a
                {
                    input.query::<&str, #mapper_ty>(#stmt, &[]).await
                }
            }
        }
    }

    pub mod count_generators {
        use super::*;
        use proc_macro2::TokenStream;

        pub fn create_count_macro(stmt: &str) -> TokenStream {
            let mssql_arm = get_mssql_arm_tokens_if_enabled(stmt, false);

            quote! {
                async fn count() -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
                    let default_db_conn = canyon_sql::core::Canyon::instance()?.get_default_connection()?;
                    let db_type = default_db_conn.get_database_type()?;
                    match db_type {
                        #mssql_arm
                        _ => {
                            default_db_conn.query_one_for::<i64>(#stmt, &[]).await
                        }
                    }
                }
            }
        }

        pub fn create_count_with_macro(stmt: &str) -> TokenStream {
            let mssql_arm = get_mssql_arm_tokens_if_enabled(stmt, true);

            quote! {
                async fn count_with<'a, I>(input: I)
                    -> Result<i64, Box<dyn std::error::Error + Send + Sync + 'a>>
                where
                    I: canyon_sql::connection::DbConnection + Send + 'a
                {
                    let db_type = input.get_database_type()?;
                    match db_type {
                        #mssql_arm
                        _ => {
                            // PostgreSQL and MySQL COUNT(*) return i64
                            input.query_one_for::<i64>(#stmt, &[]).await
                        }
                    }
                }
            }
        }

        fn get_mssql_arm_tokens_if_enabled(stmt: &str, is_with_input: bool) -> TokenStream {
            let db_conn = if is_with_input {
                quote! {input}
            } else {
                quote! {default_db_conn}
            };
            if cfg!(feature = "mssql") {
                quote! {
                    canyon_sql::connection::DatabaseType::SqlServer => {
                        let count_i32: i32 = #db_conn.query_one_for::<i32>(#stmt, &[]).await?;
                        Ok(count_i32 as i64)
                    }
                }
            } else {
                quote! {} // nothing emitted
            }
        }
    }

    pub mod find_by_pk_generators {
        use super::*;
        use crate::query_operations::consts;
        use proc_macro2::TokenStream;

        pub fn create_find_by_pk_macro(
            mapper_ty: &Ident,
            base_body: &Option<TokenStream>,
        ) -> TokenStream {
            let body = if let Some(body) = base_body {
                let default_db_conn_call = consts::generate_default_db_conn_tokens();
                quote! {
                    #body;
                    #default_db_conn_call
                        .query_one::<#mapper_ty>(&stmt, &[value])
                        .await
                }
            } else {
                let unsupported_op_err = consts::generate_no_pk_error();
                quote! { #unsupported_op_err }
            };

            quote! {
                async fn find_by_pk<'canyon_lt, 'err_lt>(value: &'canyon_lt dyn canyon_sql::query::QueryParameter)
                    -> Result<Option<#mapper_ty>, Box<(dyn std::error::Error + Send + Sync + 'err_lt)>>
                {
                    #body
                }
            }
        }

        pub fn create_find_by_pk_with(
            mapper_ty: &Ident,
            base_body: &Option<TokenStream>,
        ) -> TokenStream {
            let body = if let Some(body) = base_body {
                quote! {
                    #body;
                    input.query_one::<#mapper_ty>(&stmt, &[value]).await
                }
            } else {
                let unsupported_op_err = consts::generate_no_pk_error();
                quote! { #unsupported_op_err }
            };

            quote! {
                async fn find_by_pk_with<'canyon_lt, 'err_lt, I>(value: &'canyon_lt dyn canyon_sql::query::QueryParameter, input: I)
                    -> Result<Option<#mapper_ty>, Box<(dyn std::error::Error + Send + Sync + 'err_lt)>>
                where
                    I: canyon_sql::connection::DbConnection + Send + 'canyon_lt
                {
                    #body
                }
            }
        }
    }
}

#[cfg(test)]
mod macro_builder_read_ops_tests {
    use super::__details::{count_generators::*, find_all_generators::*};

    use crate::query_operations::consts;
    use crate::query_operations::read::__details::find_by_pk_generators::{
        create_find_by_pk_macro, create_find_by_pk_with,
    };
    use proc_macro2::Ident;

    const SELECT_ALL_STMT: &str = "SELECT * FROM public.user"; // TODO: introduce the const_format crate
    const COUNT_STMT: &str = "SELECT COUNT(*) FROM public.user";
    const FIND_BY_PK_STMT: &str = "SELECT * FROM public.user WHERE id = $1";

    #[test]
    fn test_create_find_all_macro() {
        let mapper_ty = syn::parse_str::<Ident>("User").unwrap();
        let tokens = create_find_all_macro(&mapper_ty, SELECT_ALL_STMT);
        let generated = tokens.to_string();

        assert!(generated.contains("async fn find_all"));
        assert!(generated.contains(consts::RES_RET_TY));
        assert!(generated.contains(SELECT_ALL_STMT));
    }

    #[test]
    fn test_create_find_all_with_macro() {
        let mapper_ty = syn::parse_str::<Ident>("User").unwrap();
        let tokens = create_find_all_with_macro(&mapper_ty, SELECT_ALL_STMT);
        let generated = tokens.to_string();

        assert!(generated.contains("async fn find_all_with"));
        assert!(generated.contains(consts::RES_RET_TY));
        assert!(generated.contains(consts::LT_CONSTRAINT));
        assert!(generated.contains(consts::INPUT_PARAM));
        assert!(generated.contains(SELECT_ALL_STMT));
    }

    #[test]
    fn test_create_count_macro() {
        let tokens = create_count_macro(COUNT_STMT);
        let generated = tokens.to_string();

        assert!(generated.contains("async fn count"));
        assert!(generated.contains(consts::I64_RET_TY));
        assert!(generated.contains(COUNT_STMT));
    }

    #[test]
    fn test_create_count_with_macro() {
        let tokens = create_count_with_macro(COUNT_STMT);
        let generated = tokens.to_string();

        assert!(generated.contains("async fn count_with"));
        assert!(generated.contains(consts::I64_RET_TY_LT));
        assert!(generated.contains(COUNT_STMT));
        assert!(generated.contains(consts::LT_CONSTRAINT));
        assert!(generated.contains(consts::INPUT_PARAM));
    }

    #[test]
    fn test_create_find_by_pk_macro() {
        let mapper_ty = syn::parse_str::<Ident>("User").unwrap();
        let tokens = create_find_by_pk_macro(&mapper_ty, &None);
        let generated = tokens.to_string();

        assert!(generated.contains("async fn find_by_pk"));
        assert!(generated.contains(consts::OPT_RET_TY_LT));
        assert!(generated.contains(FIND_BY_PK_STMT));
    }

    #[test]
    fn test_create_find_by_pk_with_macro() {
        let mapper_ty = syn::parse_str::<Ident>("User").unwrap();
        let tokens = create_find_by_pk_with(&mapper_ty, &None);
        let generated = tokens.to_string();

        assert!(generated.contains("async fn find_by_pk_with"));
        assert!(generated.contains(consts::OPT_RET_TY_LT));
        assert!(generated.contains(consts::LT_CONSTRAINT));
        assert!(generated.contains(FIND_BY_PK_STMT));
    }
}
