use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::TokenStream;
use quote::quote;
use canyon_core::query::querybuilder::TableMetadata;

pub fn generate_insert_tokens(
    macro_data: &MacroTokens,
    table_schema_data: &TableMetadata,
) -> TokenStream {
    let insert_method_ops = generate_insert_method_tokens(macro_data, table_schema_data);
    let insert_entity_ops = generate_insert_entity_function_tokens(table_schema_data);
    // let multi_insert_tokens = generate_multiple_insert_tokens(macro_data, table_schema_data);

    quote! {
        #insert_method_ops
        #insert_entity_ops
        // #multi_insert_tokens
    }
}

// Generates the TokenStream for the _insert operation
pub fn generate_insert_method_tokens(
    macro_data: &MacroTokens,
    table_schema_data: &str
) -> TokenStream {
    let insert_signature = quote! {
        async fn insert<'a>(&mut self)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'a>>
    };
    let insert_with_signature = quote! {
        async fn insert_with<'a, I>(&mut self, input: I)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'a>>
        where
            I: canyon_sql::connection::DbConnection + Send + 'a
    };

    let insert_body;
    let insert_with_body;
    let insert_values;

    let is_mapper_ty_present = macro_data.retrieve_mapping_target_type().is_some();
    if is_mapper_ty_present {
        let raised_err = __details::generate_unsupported_operation_err();
        insert_body = raised_err.clone(); // TODO: Can't we do it better?
        insert_with_body = raised_err;
        insert_values = quote! {};
    } else {
        let stmt = __details::generate_insert_sql_statement(macro_data, table_schema_data);
        insert_values = __details::generate_insert_fn_values_slice_expr(macro_data);
        insert_body = __details::generate_insert_fn_body_tokens(macro_data, &stmt, false);
        insert_with_body = __details::generate_insert_fn_body_tokens(macro_data, &stmt, true);
    };

    quote! {
        #insert_signature {
            #insert_values
            #insert_body
        }

        #insert_with_signature {
            #insert_values
            #insert_with_body
        }
    }
}

pub fn generate_insert_entity_function_tokens(table_schema_data: &TableMetadata,) -> TokenStream {
    let insert_entity_signature = quote! {
        async fn insert_entity<'canyon_lt, 'err_lt, Entity>(entity: &'canyon_lt mut Entity)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
        where Entity: canyon_sql::core::RowMapper
            + canyon_sql::query::bounds::Inspectionable<'canyon_lt>
            + Sync
            + 'canyon_lt
    };

    let insert_entity_with_signature = quote! {
        async fn insert_entity_with<'canyon_lt, 'err_lt, Entity, Input>(entity: &'canyon_lt mut Entity, input: Input)
            -> Result<(), Box<dyn std::error::Error + Send + Sync + 'err_lt>>
        where
            Entity: canyon_sql::core::RowMapper
                + canyon_sql::query::bounds::Inspectionable<'canyon_lt>
                + Sync
                + 'canyon_lt,
            Input: canyon_sql::connection::DbConnection + Send + 'canyon_lt
    };

    let no_fields_to_insert_err = __details::no_fields_to_insert_err();

    let stmt_ctr = quote! {
        let insert_columns = entity.fields_as_comma_sep_string();

        if insert_columns.is_empty() {
            return #no_fields_to_insert_err;
        }
        let values = entity.fields_actual_values();
        let placeholders = entity.queries_placeholders();

        let mut stmt = format!( // TODO: use the InsertQueryBuilder when created ;)
            "INSERT INTO {} ({}) VALUES ({})",
            #table_schema_data, insert_columns, placeholders
        );
    };
    let add_returning_clause = quote! {
        stmt.push_str(" RETURNING ");
        stmt.push_str(pk);
    };

    quote! {
        #insert_entity_signature {
            let default_db_conn = canyon_sql::core::Canyon::instance()?.get_default_connection()?;
            #stmt_ctr;

            if let Some(pk) = entity.primary_key() {
                #add_returning_clause
                let pk = default_db_conn.query_one_for::<<Entity as canyon_sql::query::bounds::Inspectionable>::PrimaryKeyType>(&stmt, &values).await?;
                entity.set_primary_key_actual_value(pk)?;
            } else {
                let _ = default_db_conn.execute(&stmt, &values).await?;
            }
            Ok(())
        }

        #insert_entity_with_signature {
            #stmt_ctr;
            if let Some(pk) = entity.primary_key() {
                #add_returning_clause
                let pk = input.query_one_for::<<Entity as canyon_sql::query::bounds::Inspectionable>::PrimaryKeyType>(&stmt, &values).await?;
                entity.set_primary_key_actual_value(pk)?;
            } else {
                let _ = input.execute(&stmt, &values).await?;
            }
            Ok(())
        }
    }
}

mod __details {
    use super::*;

    pub(crate) fn generate_insert_fn_body_tokens(
        macro_data: &MacroTokens,
        stmt: &str,
        is_with_method: bool,
    ) -> TokenStream {
        let pk_ident_and_type = macro_data.get_primary_key_ident_and_type();

        let db_conn = if is_with_method {
            quote! { input }
        } else {
            quote! { default_db_conn }
        };

        let mut insert_body_tokens = TokenStream::new();
        if !is_with_method {
            insert_body_tokens.extend(quote! {
                let default_db_conn = canyon_sql::core::Canyon::instance()?
                    .get_default_connection()?;
            });
        }

        if let Some(pk_data) = pk_ident_and_type {
            let pk_ident = pk_data.0;
            let pk_type = pk_data.1;

            insert_body_tokens.extend(quote! {
                self.#pk_ident = #db_conn.query_one_for::<#pk_type>(#stmt, values).await?;
                Ok(())
            });
        } else {
            insert_body_tokens.extend(quote! {
                let _ = #db_conn.execute(#stmt, values).await?;
                Ok(())
            });
        }

        insert_body_tokens
    }

    pub(crate) fn generate_insert_fn_values_slice_expr(macro_data: &MacroTokens) -> TokenStream {
        // Retrieves the fields of the Struct
        let fields = macro_data.get_columns_pk_parsed();

        let insert_values = fields.map(|field| {
            let field = field.ident.as_ref().unwrap();
            quote! { &self.#field }
        });

        quote! {
            let values: &[&dyn canyon_sql::query::QueryParameter] = &[#(#insert_values),*];
        }
    }

    pub(crate) fn generate_insert_sql_statement(
        macro_data: &MacroTokens,
        table_schema_data: &str
    ) -> String {
        // Retrieves the fields of the Struct as a collection of Strings, already parsed
        // the condition of remove the primary key if it's present, and it's auto incremental
        let insert_columns = macro_data.get_struct_fields_as_comma_sep_string();

        // Returns a String with the generic $x placeholder for the query parameters.
        // Already takes in consideration if there's pk annotation
        let placeholders = macro_data.placeholders_generator();

        let mut stmt = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table_schema_data, insert_columns, placeholders
        );

        if let Some(primary_key) = macro_data.get_primary_key_annotation() {
            stmt.push_str(format!(" RETURNING {}", primary_key).as_str());
        }

        stmt
    }

    pub(crate) fn generate_unsupported_operation_err() -> TokenStream {
        quote! {
            Err(
                std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Can't use the 'Insert' family transactions as a method (that receives self as first parameter) \
                    if your T type in CrudOperations is NOT the same type that implements RowMapper. \
                    Consider to use instead the provided insert_entity or insert_entity_with functions."
                ).into_inner().unwrap()
            )
        }
    }

    pub(crate) fn no_fields_to_insert_err() -> TokenStream {
        quote! {
            Err(
                std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "The type has either zero fields or exactly one that is annotated with #[primary_key].\
                     That's makes it ineligibly to be used in the insert_entity family of operations."
                ).into_inner().unwrap()
            )
        }
    }
}

/// Generates the TokenStream for the __insert() CRUD operation, but being available
/// as a [`QueryBuilder`] object, and instead of being a method over some [`T`] type,
/// as an associated function for [`T`]
///
/// This, also lets the user have the option to be able to insert multiple
/// [`T`] objects in only one query
fn _generate_multiple_insert_tokens(
    macro_data: &MacroTokens,
    table_schema_data: &str
) -> TokenStream {
    let ty = macro_data.ty;
    let (_, ty_generics, _) = macro_data.generics.split_for_impl();

    // Retrieves the fields of the Struct as continuous String
    let column_names = macro_data.get_struct_fields_as_comma_sep_string();

    // Retrieves the fields of the Struct
    let fields = macro_data.get_struct_fields();

    let macro_fields: Vec<TokenStream> = fields.map(|field| quote! { &instance.#field }).collect();
    let macro_fields_cloned = macro_fields.clone();

    let pk = macro_data.get_primary_key_annotation().unwrap_or_default();

    let pk_ident_type = macro_data
        .fields_with_types()
        .into_iter()
        .find(|(i, _t)| *i == &pk);

    let multi_insert_transaction = if let Some(pk_data) = &pk_ident_type {
        let pk_ident = &pk_data.0;
        let pk_type = &pk_data.1;

        quote! {
            mapped_fields = #column_names
                .split(", ")
                .map( |column_name| format!("\"{}\"", column_name))
                .collect::<Vec<String>>()
                .join(", ");

            let mut split = mapped_fields.split(", ")
                .collect::<Vec<&str>>();

            mapped_fields = #column_names
                .split(", ")
                .map( |column_name| format!("\"{}\"", column_name))
                .collect::<Vec<String>>()
                .join(", ");

            let mut split = mapped_fields.split(", ")
                .collect::<Vec<&str>>();

            let pk_value_index = split.iter()
                .position(|pk| *pk == format!("\"{}\"", #pk).as_str())
                .unwrap(); // ensured that is there
            split.retain(|pk| *pk != format!("\"{}\"", #pk).as_str());
            mapped_fields = split.join(", ").to_string();

            let mut fields_placeholders = String::new();

            let mut elements_counter = 0;
            let mut values_counter = 1;
            let values_arr_len = final_values.len();

            for vector in final_values.iter_mut() {
                let mut inner_counter = 0;
                fields_placeholders.push('(');
                vector.remove(pk_value_index);

                for _value in vector.iter() {
                    if inner_counter < vector.len() - 1 {
                        fields_placeholders.push_str(&("$".to_owned() + &values_counter.to_string() + ","));
                    } else {
                        fields_placeholders.push_str(&("$".to_owned() + &values_counter.to_string()));
                    }

                    inner_counter += 1;
                    values_counter += 1;
                }

                elements_counter += 1;

                if elements_counter < values_arr_len {
                    fields_placeholders.push_str("), ");
                } else {
                    fields_placeholders.push(')');
                }
            }

            let stmt = format!(
                "INSERT INTO {} ({}) VALUES {} RETURNING {}",
                #table_schema_data,
                mapped_fields,
                fields_placeholders,
                #pk
            );

            let mut v_arr = Vec::new();
            for arr in final_values.iter() {
                for value in arr {
                    v_arr.push(*value)
                }
            }

            let multi_insert_result = <#ty #ty_generics as canyon_sql::core::Transaction>::query_rows(
                stmt,
                v_arr,
                input
            ).await?;

            match multi_insert_result {
                #[cfg(feature="postgres")]
                canyon_sql::core::CanyonRows::Postgres(mut v) => {
                    for (idx, instance) in instances.iter_mut().enumerate() {
                        instance.#pk_ident = v
                            .get(idx)
                            .expect("Failed getting the returned IDs for a multi insert")
                            .get::<&str, #pk_type>(#pk);
                    }

                    Ok(())
                },
                #[cfg(feature="mssql")]
                canyon_sql::core::CanyonRows::Tiberius(mut v) => {
                    for (idx, instance) in instances.iter_mut().enumerate() {
                        instance.#pk_ident = v
                            .get(idx)
                            .expect("Failed getting the returned IDs for a multi insert")
                            .get::<#pk_type, &str>(#pk)
                            .expect("SQL Server primary key type failed to be set as value");
                    }

                    Ok(())
                },
                #[cfg(feature="mysql")]
                canyon_sql::core::CanyonRows::MySQL(mut v) => {
                    for (idx, instance) in instances.iter_mut().enumerate() {
                        instance.#pk_ident = v
                            .get(idx)
                            .expect("Failed getting the returned IDs for a multi insert")
                            .get::<#pk_type,usize>(0)
                            .expect("MYSQL primary key type failed to be set as value");
                    }
                    Ok(())
                },
                _ => panic!() // TODO remove when the generics will be refactored
            }
        }
    } else {
        quote! {
            mapped_fields = #column_names
                .split(", ")
                .map( |column_name| format!("\"{}\"", column_name))
                .collect::<Vec<String>>()
                .join(", ");

            let mut split = mapped_fields.split(", ")
                .collect::<Vec<&str>>();

            let mut fields_placeholders = String::new();

            let mut elements_counter = 0;
            let mut values_counter = 1;
            let values_arr_len = final_values.len();

            for vector in final_values.iter_mut() {
                let mut inner_counter = 0;
                fields_placeholders.push('(');

                for _value in vector.iter() {
                    if inner_counter < vector.len() - 1 {
                        fields_placeholders.push_str(&("$".to_owned() + &values_counter.to_string() + ","));
                    } else {
                        fields_placeholders.push_str(&("$".to_owned() + &values_counter.to_string()));
                    }

                    inner_counter += 1;
                    values_counter += 1;
                }

                elements_counter += 1;

                if elements_counter < values_arr_len {
                    fields_placeholders.push_str("), ");
                } else {
                    fields_placeholders.push(')');
                }
            }

            let stmt = format!(
                "INSERT INTO {} ({}) VALUES {}",
                #table_schema_data,
                mapped_fields,
                fields_placeholders
            );

            let mut v_arr = Vec::new();
            for arr in final_values.iter() {
                for value in arr {
                    v_arr.push(*value)
                }
            }

            <#ty #ty_generics as canyon_sql::core::Transaction>::query_rows(
                stmt,
                v_arr,
                input
            ).await?;

            Ok(())
        }
    };

    quote! {
        /// Inserts multiple instances of some type `T` into its related table.
        ///
        /// ```
        /// let mut new_league = League {
        ///     id: Default::default(),
        ///    ext_id: 392489032,
        ///     slug: "League10".to_owned(),
        ///     name: "League10also".to_owned(),
        ///     region: "Turkey".to_owned(),
        ///     image_url: "https://www.sdklafjsd.com".to_owned()
        /// };
        /// let mut new_league2 = League {
        ///     id: Default::default(),
        ///     ext_id: 392489032,
        ///     slug: "League11".to_owned(),
        ///     name: "League11also".to_owned(),
        ///     region: "LDASKJF".to_owned(),
        ///     image_url: "https://www.sdklafjsd.com".to_owned()
        /// };
        /// let mut new_league3 = League {
        ///     id: Default::default(),
        ///    ext_id: 9687392489032,
        ///     slug: "League3".to_owned(),
        ///     name: "3League".to_owned(),
        ///    region: "EU".to_owned(),
        ///     image_url: "https://www.lag.com".to_owned()
        ///};
        ///
        /// League::insert_multiple(
        ///     &mut [&mut new_league, &mut new_league2, &mut new_league3]
        /// ).await
        ///.ok();
        /// ```
         async fn multi_insert<'a, T>(instances: &'a mut [&'a mut T]) -> (
             Result<(), Box<dyn std::error::Error + Sync + std::marker::Send + 'a>>
         ) {
             let input = "";

              let mut final_values: Vec<Vec<&dyn canyon_sql::query::QueryParameter>> = Vec::new();
              for instance in instances.iter() {
                  let intermediate: &[&dyn canyon_sql::query::QueryParameter] = &[#(#macro_fields),*];

                  let mut longer_lived: Vec<&dyn canyon_sql::query::QueryParameter> = Vec::new();
                  for value in intermediate.into_iter() {
                      longer_lived.push(*value)
                  }

                  final_values.push(longer_lived)
             }

             let mut mapped_fields: String = String::new();

             #multi_insert_transaction
         }

        /// Inserts multiple instances of some type `T` into its related table with the specified
        /// datasource by its `datasource name`, defined in the configuration file.
        ///
        /// ```
        /// let mut new_league = League {
        ///     id: Default::default(),
        ///    ext_id: 392489032,
        ///     slug: "League10".to_owned(),
        ///     name: "League10also".to_owned(),
        ///     region: "Turkey".to_owned(),
        ///     image_url: "https://www.sdklafjsd.com".to_owned()
        /// };
        /// let mut new_league2 = League {
        ///     id: Default::default(),
        ///     ext_id: 392489032,
        ///     slug: "League11".to_owned(),
        ///     name: "League11also".to_owned(),
        ///     region: "LDASKJF".to_owned(),
        ///     image_url: "https://www.sdklafjsd.com".to_owned()
        /// };
        /// let mut new_league3 = League {
        ///     id: Default::default(),
        ///     ext_id: 9687392489032,
        ///     slug: "League3".to_owned(),
        ///     name: "3League".to_owned(),
        ///     region: "EU".to_owned(),
        ///     image_url: "https://www.lag.com".to_owned()
        /// };
        ///
        /// League::insert_multiple(
        ///     &mut [&mut new_league, &mut new_league2, &mut new_league3]
        /// ).await
        /// .ok();
        /// ```
        async fn multi_insert_with<'a, T, I>(instances: &'a mut [&'a mut T], input: I) ->
            Result<(), Box<dyn std::error::Error + Sync + std::marker::Send + 'a>>
            where
                I: canyon_sql::connection::DbConnection + Send + 'a
        {
            let mut final_values: Vec<Vec<&dyn canyon_sql::query::QueryParameter>> = Vec::new();
            for instance in instances.iter() {
                let intermediate: &[&dyn canyon_sql::query::QueryParameter] = &[#(#macro_fields_cloned),*];

                let mut longer_lived: Vec<&dyn canyon_sql::query::QueryParameter> = Vec::new();
                for value in intermediate.into_iter() {
                    longer_lived.push(*value)
                }

                final_values.push(longer_lived)
            }

            let mut mapped_fields: String = String::new();

            #multi_insert_transaction
        }
    }
}
