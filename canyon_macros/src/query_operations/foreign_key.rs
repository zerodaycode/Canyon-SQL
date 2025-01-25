
// /// Generates the TokenStream for build the search by foreign key feature, also as a method instance
// /// of a T type of as an associated function of same T type, but wrapped as a Result<T, Err>, representing
// /// a possible failure querying the database, a bad or missing FK annotation or a missed ForeignKeyable
// /// derive macro on the parent side of the relation
// pub fn generate_find_by_foreign_key_tokens(
//     macro_data: &MacroTokens<'_>,
// ) -> Vec<(TokenStream, TokenStream)> {
//     let mut fk_quotes: Vec<(TokenStream, TokenStream)> = Vec::new();

//     for (field_ident, fk_annot) in macro_data.get_fk_annotations().iter() {
//         if let EntityFieldAnnotation::ForeignKey(table, column) = fk_annot {
//             let method_name = "search_".to_owned() + table;

//             // TODO this is not a good implementation. We must try to capture the
//             // related entity in some way, and compare it with something else
//             let fk_ty = database_table_name_to_struct_ident(table);

//             // Generate and identifier for the method based on the convention of "search_related_types"
//             // where types is a placeholder for the plural name of the type referenced
//             let method_name_ident =
//                 proc_macro2::Ident::new(&method_name, proc_macro2::Span::call_site());
//             let method_name_ident_with = proc_macro2::Ident::new(
//                 &format!("{}_with", &method_name),
//                 proc_macro2::Span::call_site(),
//             );
//             let quoted_method_signature: TokenStream = quote! {
//                 async fn #method_name_ident(&self) ->
//                     Result<Option<#fk_ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
//             };
//             let quoted_with_method_signature: TokenStream = quote! {
//                 async fn #method_name_ident_with<'a>(&self, input: I) ->
//                     Result<Option<#fk_ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
//             };

//             let stmt = format!(
//                 "SELECT * FROM {} WHERE {} = $1",
//                 table,
//                 format!("\"{column}\"").as_str(),
//             );
//             let result_handler = quote! {
//                 match result {
//                     n if n.len() == 0 => Ok(None),
//                     _ => Ok(Some(
//                         result.into_results::<#fk_ty>().remove(0)
//                     ))
//                 }
//             };

//             fk_quotes.push((
//                 quote! { #quoted_method_signature; },
//                 quote! {
//                     /// Searches the parent entity (if exists) for this type
//                     #quoted_method_signature {
//                         let result = <#fk_ty as canyon_sql::core::Transaction<#fk_ty>>::query(
//                             #stmt,
//                             &[&self.#field_ident as &dyn canyon_sql::core::QueryParameter<'_>],
//                             ""
//                         ).await?;

//                         #result_handler
//                     }
//                 },
//             ));

//             fk_quotes.push((
//                 quote! { #quoted_with_method_signature; },
//                 quote! {
//                     /// Searches the parent entity (if exists) for this type with the specified datasource
//                     #quoted_with_method_signature {
//                         let result = <#fk_ty as canyon_sql::core::Transaction<#fk_ty>>::query(
//                             #stmt,
//                             &[&self.#field_ident as &dyn canyon_sql::core::QueryParameter<'_>],
//                             datasource_name
//                         ).await?;

//                         #result_handler
//                     }
//                 },
//             ));
//         }
//     }

//     fk_quotes
// }

// /// Generates the TokenStream for build the __search_by_foreign_key() CRUD
// /// associated function, but wrapped as a Result<T, Err>, representing
// /// a possible failure querying the database, a bad or missing FK annotation or a missed ForeignKeyable
// /// derive macro on the parent side of the relation
// pub fn generate_find_by_reverse_foreign_key_tokens(
//     macro_data: &MacroTokens<'_>,
//     table_schema_data: &String,
// ) -> Vec<(TokenStream, TokenStream)> {
//     let mut rev_fk_quotes: Vec<(TokenStream, TokenStream)> = Vec::new();
//     let ty = macro_data.ty;

//     for (field_ident, fk_annot) in macro_data.get_fk_annotations().iter() {
//         if let EntityFieldAnnotation::ForeignKey(table, column) = fk_annot {
//             let method_name = format!("search_{table}_childrens");

//             // Generate and identifier for the method based on the convention of "search_by__" (note the double underscore)
//             // plus the 'table_name' property of the ForeignKey annotation
//             let method_name_ident =
//                 proc_macro2::Ident::new(&method_name, proc_macro2::Span::call_site());
//             let method_name_ident_with = proc_macro2::Ident::new(
//                 &format!("{}_with", &method_name),
//                 proc_macro2::Span::call_site(),
//             );
//             let quoted_method_signature: TokenStream = quote! {
//                 async fn #method_name_ident<'a, F: canyon_sql::crud::bounds::ForeignKeyable<F> + Sync + Send>(value: &F) ->
//                     Result<Vec<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
//             };
//             let quoted_with_method_signature: TokenStream = quote! {
//                 async fn #method_name_ident_with<'a, F: canyon_sql::crud::bounds::ForeignKeyable<F> + Sync + Send>
//                     (value: &F, input: I) ->
//                     Result<Vec<#ty>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
//             };

//             let f_ident = field_ident.to_string();

//             rev_fk_quotes.push((
//                 quote! { #quoted_method_signature; },
//                 quote! {
//                     /// Given a parent entity T annotated with the derive proc macro `ForeignKeyable`,
//                     /// performns a search to find the children that belong to that concrete parent.
//                     #quoted_method_signature
//                     {
//                         let lookage_value = value.get_fk_column(#column)
//                             .expect(format!(
//                                 "Column: {:?} not found in type: {:?}", #column, #table
//                             ).as_str());

//                         let stmt = format!(
//                             "SELECT * FROM {} WHERE {} = $1",
//                             #table_schema_data,
//                             format!("\"{}\"", #f_ident).as_str()
//                         );

//                         Ok(<#ty as canyon_sql::core::Transaction<#ty>>::query(
//                             stmt,
//                             &[lookage_value],
//                             ""
//                         ).await?.into_results::<#ty>())
//                     }
//                 },
//             ));

//             rev_fk_quotes.push((
//                 quote! { #quoted_with_method_signature; },
//                 quote! {
//                     /// Given a parent entity T annotated with the derive proc macro `ForeignKeyable`,
//                     /// performns a search to find the children that belong to that concrete parent
//                     /// with the specified datasource.
//                     #quoted_with_method_signature
//                     {
//                         let lookage_value = value.get_fk_column(#column)
//                             .expect(format!(
//                                 "Column: {:?} not found in type: {:?}", #column, #table
//                             ).as_str());

//                         let stmt = format!(
//                             "SELECT * FROM {} WHERE {} = $1",
//                             #table_schema_data,
//                             format!("\"{}\"", #f_ident).as_str()
//                         );

//                         Ok(<#ty as canyon_sql::core::Transaction<#ty>>::query(
//                             stmt,
//                             &[lookage_value],
//                             datasource_name
//                         ).await?.into_results::<#ty>())
//                     }
//                 },
//             ));
//         }
//     }

//     rev_fk_quotes
// }