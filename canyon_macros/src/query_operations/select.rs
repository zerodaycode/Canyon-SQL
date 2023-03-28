use canyon_observer::manager::field_annotation::EntityFieldAnnotation;

use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;
use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for build the __find_all() CRUD
/// associated function
pub fn generate_find_all_unchecked_tokens(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &String,
) -> TokenStream {
    let ty = macro_data.ty;
    let stmt = format!("SELECT * FROM {table_schema_data}");

    quote! {
        /// Performns a `SELECT * FROM table_name`, where `table_name` it's
        /// the name of your entity but converted to the corresponding
        /// database convention. P.ej. PostgreSQL prefers table names declared
        /// with snake_case identifiers.
        async fn find_all_unchecked<'a>() -> Vec<#ty> {
            <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                #stmt,
                &[],
                ""
            ).await
            .unwrap()
            .get_entities::<#ty>()
        }

        /// Performns a `SELECT * FROM table_name`, where `table_name` it's
        /// the name of your entity but converted to the corresponding
        /// database convention. P.ej. PostgreSQL prefers table names declared
        /// with snake_case identifiers.
        ///
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the [`&str`]
        /// passed as parameter.
        async fn find_all_unchecked_datasource<'a>(datasource_name: &'a str) -> Vec<#ty> {
            <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                #stmt,
                &[],
                datasource_name
            ).await
            .unwrap()
            .get_entities::<#ty>()
        }
    }
}

/// Generates the TokenStream for build the __find_all_result() CRUD
/// associated function
pub fn generate_find_all_tokens(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &String,
) -> TokenStream {
    let ty = macro_data.ty;
    let stmt = format!("SELECT * FROM {table_schema_data}");

    quote! {
        /// Performns a `SELECT * FROM table_name`, where `table_name` it's
        /// the name of your entity but converted to the corresponding
        /// database convention. P.ej. PostgreSQL prefers table names declared
        /// with snake_case identifiers.
        async fn find_all<'a>() ->
            Result<Vec<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
        {
            Ok(
                <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                    #stmt,
                    &[],
                    ""
                ).await?
                .get_entities::<#ty>()
            )
        }

        /// Performns a `SELECT * FROM table_name`, where `table_name` it's
        /// the name of your entity but converted to the corresponding
        /// database convention. P.ej. PostgreSQL prefers table names declared
        /// with snake_case identifiers.
        ///
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the [`&str`]
        /// passed as parameter.
        ///
        /// Also, returns a [`Vec<T>, Error>`], wrapping a possible failure
        /// querying the database, or, if no errors happens, a Vec<T> containing
        /// the data found.
        async fn find_all_datasource<'a>(datasource_name: &'a str) ->
            Result<Vec<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
        {
            Ok(
                <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                    #stmt,
                    &[],
                    datasource_name
                ).await?
                .get_entities::<#ty>()
            )
        }
    }
}

/// Same as above, but with a [`canyon_sql::query::QueryBuilder`]
pub fn generate_find_all_query_tokens(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &String,
) -> TokenStream {
    let ty = macro_data.ty;

    quote! {
        /// Generates a [`canyon_sql::query::SelectQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs a `SELECT * FROM  table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        fn select_query<'a>() -> canyon_sql::query::SelectQueryBuilder<'a, #ty> {
            canyon_sql::query::SelectQueryBuilder::new(#table_schema_data, "")
        }

        /// Generates a [`canyon_sql::query::SelectQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs a `SELECT * FROM  table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        ///
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the [`&str`]
        /// passed as parameter.
        fn select_query_datasource<'a>(datasource_name: &'a str) -> canyon_sql::query::SelectQueryBuilder<'a, #ty> {
            canyon_sql::query::SelectQueryBuilder::new(#table_schema_data, datasource_name)
        }
    }
}

/// Performs a COUNT(*) query over some table, returning a [`Result`] wrapping
/// a possible success or error coming from the database
pub fn generate_count_tokens(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &String,
) -> TokenStream {
    let ty = macro_data.ty;
    let ty_str = &ty.to_string();
    let stmt = format!("SELECT COUNT (*) FROM {table_schema_data}");

    let result_handling = quote! {
        match count.get_active_ds() {
            canyon_sql::crud::DatabaseType::PostgreSql => {
                Ok(
                    count.postgres.get(0)
                        .expect(&format!("Count operation failed for {:?}", #ty_str))
                        .get::<&str, i64>("count")
                        .to_owned()
                )
            },
            canyon_sql::crud::DatabaseType::SqlServer => {
                Ok(
                    count.sqlserver.get(0)
                        .expect(&format!("Count operation failed for {:?}", #ty_str))
                        .get::<i32, usize>(0)
                        .expect(&format!("SQL Server failed to return the count values for {:?}", #ty_str))
                        .into()
                )
            }
        }
    };

    quote! {
        /// Performs a COUNT(*) query over some table, returning a [`Result`] rather than panicking,
        /// wrapping a possible success or error coming from the database
        async fn count() -> Result<i64, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
            let count = <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                #stmt,
                &[],
                ""
            ).await?;

            #result_handling
        }

        /// Performs a COUNT(*) query over some table, returning a [`Result`] rather than panicking,
        /// wrapping a possible success or error coming from the database with the specified datasource
        async fn count_datasource<'a>(datasource_name: &'a str) -> Result<i64, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
            let count = <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                #stmt,
                &[],
                datasource_name
            ).await?;

            #result_handling
        }
    }
}

/// Generates the TokenStream for build the __find_by_pk() CRUD operation
pub fn generate_find_by_pk_tokens(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &String,
) -> TokenStream {
    let ty = macro_data.ty;
    let pk = macro_data.get_primary_key_annotation().unwrap_or_default();
    let stmt = format!("SELECT * FROM {table_schema_data} WHERE {pk} = $1");

    // Disabled if there's no `primary_key` annotation
    if pk.is_empty() {
        return quote! {
            async fn find_by_pk<'a>(value: &'a dyn canyon_sql::crud::bounds::QueryParameter<'a>)
                -> Result<Option<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
            {
                Err(
                    std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "You can't use the 'find_by_pk' associated function on a \
                        CanyonEntity that does not have a #[primary_key] annotation. \
                        If you need to perform an specific search, use the Querybuilder instead."
                    ).into_inner().unwrap()
                )
            }

            async fn find_by_pk_datasource<'a>(
                value: &'a dyn canyon_sql::crud::bounds::QueryParameter<'a>,
                datasource_name: &'a str
            ) -> Result<Option<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
                Err(
                    std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "You can't use the 'find_by_pk_datasource' associated function on a \
                        CanyonEntity that does not have a #[primary_key] annotation. \
                        If you need to perform an specific search, use the Querybuilder instead."
                    ).into_inner().unwrap()
                )
            }
        };
    }

    let result_handling = quote! {
        match result {
            n if n.number_of_results() == 0 => Ok(None),
            _ => Ok(
                Some(result.get_entities::<#ty>().remove(0))
            )
        }
    };

    quote! {
        /// Finds an element on the queried table that matches the
        /// value of the field annotated with the `primary_key` attribute,
        /// filtering by the column that it's declared as the primary
        /// key on the database.
        ///
        /// This operation it's only available if the [`CanyonEntity`] contains
        /// some field declared as primary key.
        ///
        /// Also, returns a [`Result<Option<T>, Error>`], wrapping a possible failure
        /// querying the database, or, if no errors happens, a success containing
        /// and Option<T> with the data found wrapped in the Some(T) variant,
        /// or None if the value isn't found on the table.
        async fn find_by_pk<'a>(value: &'a dyn canyon_sql::crud::bounds::QueryParameter<'a>) ->
            Result<Option<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
        {
            let result = <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                #stmt,
                vec![value],
                ""
            ).await?;

            #result_handling
        }

        /// Finds an element on the queried table that matches the
        /// value of the field annotated with the `primary_key` attribute,
        /// filtering by the column that it's declared as the primary
        /// key on the database.
        ///
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the [`&str`]
        /// passed as parameter.
        ///
        /// This operation it's only available if the [`CanyonEntity`] contains
        /// some field declared as primary key.
        ///
        /// Also, returns a [`Result<Option<T>, Error>`], wrapping a possible failure
        /// querying the database, or, if no errors happens, a success containing
        /// and Option<T> with the data found wrapped in the Some(T) variant,
        /// or None if the value isn't found on the table.
        async fn find_by_pk_datasource<'a>(
            value: &'a dyn canyon_sql::crud::bounds::QueryParameter<'a>,
            datasource_name: &'a str
        ) -> Result<Option<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {

            let result = <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                #stmt,
                vec![value],
                datasource_name
            ).await?;

            #result_handling
        }
    }
}

/// Generates the TokenStream for build the search by foreign key feature, also as a method instance
/// of a T type of as an associated function of same T type, but wrapped as a Result<T, Err>, representing
/// a possible failure querying the database, a bad or missing FK annotation or a missed ForeignKeyable
/// derive macro on the parent side of the relation
pub fn generate_find_by_foreign_key_tokens(
    macro_data: &MacroTokens<'_>,
) -> Vec<(TokenStream, TokenStream)> {
    let mut fk_quotes: Vec<(TokenStream, TokenStream)> = Vec::new();

    for (field_ident, fk_annot) in macro_data.get_fk_annotations().iter() {
        if let EntityFieldAnnotation::ForeignKey(table, column) = fk_annot {
            let method_name = "search_".to_owned() + table;

            // TODO this is not a good implementation. We must try to capture the
            // related entity in some way, and compare it with something else
            let fk_ty = database_table_name_to_struct_ident(table);

            // Generate and identifier for the method based on the convention of "search_related_types"
            // where types is a placeholder for the plural name of the type referenced
            let method_name_ident =
                proc_macro2::Ident::new(&method_name, proc_macro2::Span::call_site());
            let method_name_ident_ds = proc_macro2::Ident::new(
                &format!("{}_datasource", &method_name),
                proc_macro2::Span::call_site(),
            );
            let quoted_method_signature: TokenStream = quote! {
                async fn #method_name_ident(&self) ->
                    Result<Option<#fk_ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
            };
            let quoted_datasource_method_signature: TokenStream = quote! {
                async fn #method_name_ident_ds<'a>(&self, datasource_name: &'a str) ->
                    Result<Option<#fk_ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
            };

            let stmt = format!(
                "SELECT * FROM {} WHERE {} = $1",
                table,
                format!("\"{column}\"").as_str(),
            );
            let result_handler = quote! {
                match result {
                    n if n.number_of_results() == 0 => Ok(None),
                    _ => Ok(Some(
                        result.get_entities::<#fk_ty>().remove(0)
                    ))
                }
            };

            fk_quotes.push((
                quote!{ #quoted_method_signature; },
                quote! {
                    /// Searches the parent entity (if exists) for this type
                    #quoted_method_signature {
                        let result = <#fk_ty as canyon_sql::crud::Transaction<#fk_ty>>::query(
                            #stmt,
                            &[&self.#field_ident as &dyn canyon_sql::crud::bounds::QueryParameter<'_>],
                            ""
                        ).await?;

                        #result_handler
                    }
                }
            ));

            fk_quotes.push((
                quote! { #quoted_datasource_method_signature; },
                quote! {
                    /// Searches the parent entity (if exists) for this type with the specified datasource
                    #quoted_datasource_method_signature {
                        let result = <#fk_ty as canyon_sql::crud::Transaction<#fk_ty>>::query(
                            #stmt,
                            &[&self.#field_ident as &dyn canyon_sql::crud::bounds::QueryParameter<'_>],
                            datasource_name
                        ).await?;

                        #result_handler
                    }
                }
            ));
        }
    }

    fk_quotes
}

/// Generates the TokenStream for build the __search_by_foreign_key() CRUD
/// associated function, but wrapped as a Result<T, Err>, representing
/// a possible failure querying the database, a bad or missing FK annotation or a missed ForeignKeyable
/// derive macro on the parent side of the relation
pub fn generate_find_by_reverse_foreign_key_tokens(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &String,
) -> Vec<(TokenStream, TokenStream)> {
    let mut rev_fk_quotes: Vec<(TokenStream, TokenStream)> = Vec::new();
    let ty = macro_data.ty;

    for (field_ident, fk_annot) in macro_data.get_fk_annotations().iter() {
        if let EntityFieldAnnotation::ForeignKey(table, column) = fk_annot {
            let method_name = format!("search_{table}_childrens");

            // Generate and identifier for the method based on the convention of "search_by__" (note the double underscore)
            // plus the 'table_name' property of the ForeignKey annotation
            let method_name_ident =
                proc_macro2::Ident::new(&method_name, proc_macro2::Span::call_site());
            let method_name_ident_ds = proc_macro2::Ident::new(
                &format!("{}_datasource", &method_name),
                proc_macro2::Span::call_site(),
            );
            let quoted_method_signature: TokenStream = quote! {
                async fn #method_name_ident<'a, F: canyon_sql::crud::bounds::ForeignKeyable<F> + Sync + Send>(value: &F) ->
                    Result<Vec<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
            };
            let quoted_datasource_method_signature: TokenStream = quote! {
                async fn #method_name_ident_ds<'a, F: canyon_sql::crud::bounds::ForeignKeyable<F> + Sync + Send>
                    (value: &F, datasource_name: &'a str) ->
                    Result<Vec<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
            };

            let f_ident = field_ident.to_string();

            rev_fk_quotes.push((
                quote! { #quoted_method_signature; },
                quote! {
                    /// Given a parent entity T annotated with the derive proc macro `ForeignKeyable`,
                    /// performns a search to find the children that belong to that concrete parent.
                    #quoted_method_signature
                    {
                        let lookage_value = value.get_fk_column(#column)
                        .expect(format!(
                            "Column: {:?} not found in type: {:?}", #column, #table
                            ).as_str());

                        let stmt = format!(
                            "SELECT * FROM {} WHERE {} = $1",
                            #table_schema_data,
                            format!("\"{}\"", #f_ident).as_str()
                        );

                        Ok(<#ty as canyon_sql::crud::Transaction<#ty>>::query(
                            stmt,
                            &[lookage_value],
                            ""
                        ).await?
                        .get_entities::<#ty>())
                    }
                },
            ));

            rev_fk_quotes.push((
                quote! { #quoted_datasource_method_signature; },
                quote! {
                    /// Given a parent entity T annotated with the derive proc macro `ForeignKeyable`,
                    /// performns a search to find the children that belong to that concrete parent
                    /// with the specified datasource.
                    #quoted_datasource_method_signature
                    {
                        let lookage_value = value.get_fk_column(#column)
                            .expect(format!(
                                "Column: {:?} not found in type: {:?}", #column, #table
                            ).as_str());

                        let stmt = format!(
                            "SELECT * FROM {} WHERE {} = $1",
                            #table_schema_data,
                            format!("\"{}\"", #f_ident).as_str()
                        );

                        Ok(<#ty as canyon_sql::crud::Transaction<#ty>>::query(
                            stmt,
                            &[lookage_value],
                            datasource_name
                        ).await?
                        .get_entities::<#ty>())
                    }
                },
            ));
        }
    }

    rev_fk_quotes
}
