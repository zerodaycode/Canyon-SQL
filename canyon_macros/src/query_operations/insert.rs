use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;
use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the _insert() CRUD operation
pub fn generate_insert_tokens(macro_data: &MacroTokens) -> TokenStream {

    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    // Retrieves the fields of the Struct as continuous String
    let column_names = macro_data.get_struct_fields_as_strings();

    // Retrives the fields of the Struct
    let fields = macro_data.get_struct_fields();

    let insert_values = fields.iter().map( |ident| {
        quote! { &self.#ident }
    });

    quote! {
        /// Inserts into a database entity the current data in `self`, generating a new
        /// entry (row), returning the `PRIMARY KEY` = `self.id`
        /// 
        /// This `insert` operation needs a `&mut` reference. That's because typically, 
        /// an insert operation represents *new* data stored in the database, so, when
        /// inserted, the database will generate a unique new value for the mandatory 
        /// `id` field, having a unique identifier for every record, and it will
        /// automatically assign that returned id to `self.id`. So, after the `insert`
        /// operation, you instance will have the correct value that is the *PRIMARY KEY*
        /// of the database row that represents.
        /// 
        /// ## *Examples*
        /// ```
        
        /// let mut lec: League = League {
        ///     id: Default::default(),
        ///     ext_id: 1,
        ///     slug: "LEC".to_string(),
        ///     name: "League Europe Champions".to_string(),
        ///     region: "EU West".to_string(),
        ///     image_url: "https://lec.eu".to_string(),
        /// };

        /// let mut lck: League = League {
        ///     id: Default::default(),
        ///     ext_id: 2,
        ///     slug: "LCK".to_string(),
        ///     name: "League Champions Korea".to_string(),
        ///     region: "South Korea".to_string(),
        ///     image_url: "https://korean_lck.kr".to_string(),
        /// };

        /// let mut lpl: League = League {
        ///     id: Default::default(),
        ///     ext_id: 3,
        ///     slug: "LPL".to_string(),
        ///     name: "League PRO China".to_string(),
        ///     region: "China".to_string(),
        ///     image_url: "https://chinese_lpl.ch".to_string(),
        /// };

        /// Now, the insert operations in Canyon is designed as a method over
        /// the object, so the data of the instance is automatically parsed
        /// into it's correct types and formats and inserted into the table
        /// lec.insert().await;
        /// lck.insert().await;
        /// lpl.insert().await;
        /// 
        /// ## self.id
        /// Remember that after the insert operation, you instance already have 
        /// the correct value for the `self.id` field.
        /// ```
        #vis async fn insert(&mut self) -> () {
            self.id = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__insert(
                #table_name, 
                #column_names, 
                &[
                    #(#insert_values),*
                ]
            ).await
            .ok()
            .expect(
                format!(
                    "Insert operation failed for {:?}", 
                    &self
                ).as_str()
            );
        }
    }
}

/// Generates the TokenStream for the _insert_result() CRUD operation
pub fn generate_insert_result_tokens(macro_data: &MacroTokens) -> TokenStream {

    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    // Retrieves the fields of the Struct as continuous String
    let column_names = macro_data.get_struct_fields_as_strings();

    // Retrives the fields of the Struct
    let fields = macro_data.get_struct_fields();

    let insert_values = fields.iter().map( |ident| {
        quote! { &self.#ident }
    });

    quote! {
        /// Inserts into a database entity the current data in `self`, generating a new
        /// entry (row), returning the `PRIMARY KEY` = `self.id`
        /// 
        /// This `insert` operation needs a `&mut` reference. That's because typically, 
        /// an insert operation represents *new* data stored in the database, so, when
        /// inserted, the database will generate a unique new value for the mandatory 
        /// `id` field, having a unique identifier for every record, and it will
        /// automatically assign that returned id to `self.id`. So, after the `insert`
        /// operation, you instance will have the correct value that is the *PRIMARY KEY*
        /// of the database row that represents.
        /// 
        /// This operation returns a result type, indicating a posible failure querying the database.
        /// 
        /// ## *Examples*
        ///```
        /// let mut lec: League = League {
        ///     id: Default::default(),
        ///     ext_id: 1,
        ///     slug: "LEC".to_string(),
        ///     name: "League Europe Champions".to_string(),
        ///     region: "EU West".to_string(),
        ///     image_url: "https://lec.eu".to_string(),
        /// };
        ///
        /// println!("LEC before: {:?}", &lec);
        ///
        /// let ins_result = lec.insert_result().await;
        ///
        /// Now, we can handle the result returned, because it can contains a
        /// critical error that may leads your program to panic
        /// if let Ok(_) = ins_result {
        ///     println!("LEC after: {:?}", &lec);
        /// } else {
        ///     eprintln!("{:?}", ins_result.err())
        /// }
        /// ```
        /// 
        #vis async fn insert_result(&mut self) -> Result<(), canyon_sql::tokio_postgres::Error> {
            let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__insert(
                #table_name, 
                #column_names, 
                &[
                    #(#insert_values),*
                ]
            ).await;

            if let Err(error) = result {
                Err(error)
            } else {
                self.id = result  
                    .ok()
                    .expect(
                        format!(
                            "Insert operation failed for {:?}", 
                            &self
                        ).as_str()
                    );

                Ok(())
            }
        }
    }
}

/// Generates the TokenStream for the __insert() CRUD operation, but being available
/// as a [`QueryBuilder`] object, and instead of being a method over some [`T`] type, 
/// as an associated function for [`T`]
/// 
/// This, also lets the user to have the option to be able to insert multiple
/// [`T`] objects in only one query
pub fn generate_multiple_insert_tokens(macro_data: &MacroTokens) -> TokenStream {

    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    // Retrieves the fields of the Struct as continuous String
    let column_names = macro_data.get_struct_fields_as_strings();
    
    // Retrives the fields of the Struct
    let fields = macro_data.get_struct_fields();
    
    let macro_fields = fields.iter().map( |field| 
        quote! {
            &instance.#field 
        } 
    );


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
        #vis async fn insert_multiple(values: &mut [&mut #ty]) -> (
            Result<(), canyon_sql::tokio_postgres::Error> 
        ) {
            use crate::tokio_postgres::types::ToSql;
            
            let mut final_values: Vec<Vec<Box<&(dyn ToSql + Sync)>>> = Vec::new();
            for instance in values.iter() {
                let intermediate: &[&(dyn ToSql + Sync)] = &[#(#macro_fields),*];
                
                let mut longer_lived: Vec<Box<&(dyn ToSql + Sync)>> = Vec::new();
                for value in intermediate.iter() {
                    longer_lived.push(Box::new(*value))
                }

                final_values.push(longer_lived)
            }
            
            let autogenerated_ids = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__insert_multi(
                #table_name, 
                #column_names, 
                &mut final_values
            ).await;

            if let Err(error) = autogenerated_ids {
                Err(error)
            } else {
                for (idx, instance) in values.iter_mut().enumerate() {
                    instance.id = autogenerated_ids
                        .as_ref()
                        .ok()
                        .unwrap()
                        .wrapper
                        .get(idx)
                        .expect("Failed getting the returned ID for an insert")
                        .get("id");
                }

                Ok(())
            }
        }
    }
}