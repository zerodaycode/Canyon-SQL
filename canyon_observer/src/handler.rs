use std::{any::{type_name_of_val, TypeId}};

use canyon_connection::{CACHED_DATABASE_CONN, get_database_type_from_datasource_name, tiberius::{self, ColumnData}};
use partialdebug::placeholder::PartialDebug;
use tokio_postgres::{types::Type};

use canyon_crud::{crud::Transaction, bounds::{Row, RowOperations, Column}, DatabaseType, result::DatabaseResult};

use crate::{constants, postgresql::information_schema::{information_schema_row_mapper::ColumnTypeValue, rows_to_table_mapper::{TableMetadata, ColumnMetadata}}};

use super::{
    memory::CanyonMemory,
    postgresql::{
        migrations::DatabaseSyncOperations,
    },
    CANYON_REGISTER_ENTITIES,
};

#[derive(PartialDebug)]
pub struct Migrations;
// Makes this structure able to make queries to the database
impl Transaction<Self> for Migrations {}

impl Migrations {
    /// Launches the mechanism to parse the Database schema, the Canyon register
    /// and the database table with the memory of Canyon to perform the
    /// migrations over the targeted database
    pub async fn migrate(datasource_name: &str) {
        let mut db_operation = DatabaseSyncOperations::default();
        let canyon_tables = CANYON_REGISTER_ENTITIES.lock().unwrap().to_vec();

        // Tracked entities that must be migrated whenever Canyon starts
        let db_type = get_database_type_from_datasource_name(datasource_name).await;
        let schema_status = Self::fetch_database(datasource_name, db_type).await;
        let table_rows = Self::map_rows(&schema_status);
    }

    /// Fetches a concrete schema metadata by target the database
    /// choosed by it's datasource name property
    async fn fetch_database(datasource_name: &str, db_type: DatabaseType) -> DatabaseResult<Migrations>
    {
        let query = match db_type {
            DatabaseType::PostgreSql => constants::postgresql_queries::FETCH_PUBLIC_SCHEMA, 
            DatabaseType::SqlServer => todo!()
        };

        Self::query(query, &[], datasource_name)
            .await
            .unwrap()
    }

    ///
    fn map_rows(db_results: &DatabaseResult<Migrations>) -> Vec<TableMetadata> {
        let mut schema_info: Vec<TableMetadata> = Vec::new();

        for res_row in db_results.as_canyon_row().into_iter() {
            println!("\nROW: {:?}", &res_row.as_any().downcast_ref::<tokio_postgres::Row>());
            println!("Table name from row: {}", res_row.get::<&str>("table_name"));
            let unique_table = schema_info
                .iter_mut()
                .find(|table| (*table).table_name == res_row.get::<&str>("table_name").to_owned());
            match unique_table {
                Some(table) => {
                    /* If a table entity it's already present on the collection, we add it
                    the founded columns related to the table */
                    println!("\nParsing unique_table: {}", table.table_name);
                    println!("Table name from row: {}", res_row.get::<&str>("table_name"));
                    Self::get_columns_metadata(res_row, table);
                }
                None => {
                    /* If there's no table for a given "table_name" property on the
                    collection yet, we must create a new instance and attach it
                    the founded columns data in this iteration */
                    let mut new_table = TableMetadata {
                        table_name: res_row.get::<&str>("table_name").to_owned(),
                        columns: Vec::new(),
                    };
                    println!("\nParsing old_table: {}", new_table.table_name);
                    Self::get_columns_metadata(res_row, &mut new_table);
                    schema_info.push(new_table);
                }
            };
        }

        // println!("SCHEMA INFO [0]: {:?}", &schema_info.get(0));
        // println!("\nSCHEMA INFO [1]: {:?}", &schema_info.get(1));
        println!("\nROW [0]: {:?}", &db_results.postgres.get(0));
        schema_info
    }

    /// Maps a [`Row`] from the `information_schema` table into a [`TableMetadata`],
    /// by extracting every single column in a Row and making a relation between the column's name,
    /// the datatype of the value that it's holding, and the value itself.
    fn get_columns_metadata(res_row: &dyn Row, table: &mut TableMetadata) {
        for column in res_row.columns().iter() {
            if column.name() != "table_name" { // Discards the column "table_name"
                table.columns.push(Self::set_column_metadata(res_row, column));
            }
        }
    }

    /// 
    fn set_column_metadata<'a>(row: &dyn Row, column: &Column) -> ColumnMetadata<'a> {
        let mut entity_column = ColumnMetadata::default();
        let column_identifier = column.name();
        let column_value = ColumnTypeValue::get_value(row, column);
        println!("COL - Parsing: {} with value => {:?}", &column_identifier, &column_value);

        if column_identifier == "column_name" {
            if let ColumnTypeValue::StringValue(value) = &column_value {
                entity_column.column_name = value.to_owned().unwrap()
            }
        } else if column_identifier == "data_type" {
            if let ColumnTypeValue::StringValue(value) = &column_value {
                entity_column.postgres_datatype = value.to_owned().unwrap()
            }
        } else if column_identifier == "character_maximum_length" {
            if let ColumnTypeValue::IntValue(value) = &column_value {
                entity_column.character_maximum_length = value.to_owned()
            }
        } else if column_identifier == "is_nullable" {
            if let ColumnTypeValue::StringValue(value) = &column_value {
                entity_column.is_nullable = matches!(value.as_ref().unwrap().as_str(), "YES")
            }
        } else if column_identifier == "column_default" {
            if let ColumnTypeValue::StringValue(value) = &column_value {
                entity_column.column_default = value.to_owned()
            }
        } else if column_identifier == "numeric_precision" {
            if let ColumnTypeValue::IntValue(value) = &column_value {
                entity_column.numeric_precision = value.to_owned()
            }
        } else if column_identifier == "numeric_scale" {
            if let ColumnTypeValue::IntValue(value) = &column_value {
                entity_column.numeric_scale = value.to_owned()
            }
        } else if column_identifier == "numeric_precision_radix" {
            if let ColumnTypeValue::IntValue(value) = &column_value {
                entity_column.numeric_precision_radix = value.to_owned()
            }
        } else if column_identifier == "datetime_precision" {
            if let ColumnTypeValue::IntValue(value) = &column_value {
                entity_column.datetime_precision = value.to_owned()
            }
        } else if column_identifier == "interval_type" {
            if let ColumnTypeValue::StringValue(value) = &column_value {
                entity_column.interval_type = value.to_owned()
            }
        } else if column_identifier == "foreign_key_info" {
            if let ColumnTypeValue::StringValue(value) = &column_value {
                entity_column.foreign_key_info = value.to_owned()
            }
        } else if column_identifier == "foreign_key_name" {
            if let ColumnTypeValue::StringValue(value) = &column_value {
                entity_column.foreign_key_name = value.to_owned()
            }
        } else if column_identifier == "primary_key_info" {
            if let ColumnTypeValue::StringValue(value) = &column_value {
                entity_column.primary_key_info = value.to_owned()
            }
        } else if column_identifier == "primary_key_name" {
            if let ColumnTypeValue::StringValue(value) = &column_value {
                entity_column.primary_key_name = value.to_owned()
            }
        } else if column_identifier == "is_identity" {
            if let ColumnTypeValue::StringValue(value) = &column_value {
                entity_column.is_identity = matches!(value.as_ref().unwrap().as_str(), "YES")
            }
        } else if column_identifier == "identity_generation" {
            if let ColumnTypeValue::StringValue(value) = &column_value {
                entity_column.identity_generation = value.to_owned()
            }
        };

        entity_column
        // Just for split the related column data into what will be the values for
        // every DatabaseTableColumn.
        // Every time that we find an &RelatedColumn which column identifier
        // is == "identity_generation", we know that we finished to set the values
        // for a new DatabaseTableColumn
        // if &column.column_identifier == "identity_generation" {
        //     table_entity.columns.push(entity_column.clone());
        //     if idx == mapped_table.columns.len() - 1 {
        //         entity_column = DatabaseTableColumn::new();
        //     }
        // }
    }
}

/// Provides the necessary entities to let Canyon perform and develop
/// it's full potential, completly managing all the entities written by
/// the user and annotated with the `#[canyon_entity]`
#[derive(PartialDebug)]
pub struct CanyonHandler;

// Makes this structure able to make queries to the database
impl Transaction<Self> for CanyonHandler {}

impl CanyonHandler {
    // pub async fn run() {
    //     let mut db_operation = DatabaseSyncOperations::default();
    //     let canyon_tables = CANYON_REGISTER_ENTITIES.lock().unwrap().to_vec();
    //     db_operation
    //         .fill_operations(
    //             CanyonMemory::remember().await,
    //             canyon_tables,
    //             Self::fetch_postgres_database_status().await,
    //         )
    //         .await;
    // }

    /*
    Fetches the *information schema* of the *public schema* of a `PostgreSQL` database,
    in order to retrieve the relation between the tables and it's columns, constraints
    configurations...

    ```ignore
    table_name      column_name     data_type           is_nullable
    ---------------------------------------------------------------
    canyon_memory   filename        character varying   NO
    canyon_memory   id              integer             NO
    canyon_memory   struct_name     character varying   NO
    league          ext_id          bigint              YES
    league          id              integer             NO
    league          image_url       text                YES
    league          name            text                YES
    league          region          text                YES
    league          slug            text                YES
    tournament      end_date        date                YES
    tournament      ext_id          bigint              YES
    tournament      id              integer             NO
    tournament      league          integer             YES
    tournament      slug            text                YES
    tournament      start_date      date                YES
    ```
    > Not all retrieved columns are included in the example above.

    Too see all the columns that will be mapeed, see the [`struct@RowTable`]
    */
    // async fn fetch_postgres_database_status<'b>() -> Vec<DatabaseTable<'b>> {
    //     let results = Self::query(
    //         super::constants::postgresql_queries::FETCH_PUBLIC_SCHEMA,
    //         vec![],
    //         "",
    //     ).await
    //     .ok()
    //     .unwrap()
    //     .postgres;

    //     let mut schema_info: Vec<RowTable> = Vec::new();

    //     for res_row in results.iter() {
    //         let unique_table = schema_info
    //             .iter_mut()
    //             .find(|table| table.table_name == res_row.get::<&str, String>("table_name"));
    //         match unique_table {
    //             Some(table) => {
    //                 /* If a table entity it's already present on the collection, we add it
    //                 the founded columns related to the table */
    //                 Self::get_row_postgres_columns_for_table(res_row, table);
    //             }
    //             None => {
    //                 /* If there's no table for a given "table_name" property on the
    //                 collection yet, we must create a new instance and attach it
    //                 the founded columns data in this iteration */
    //                 let mut new_table = RowTable {
    //                     table_name: res_row.get::<&str, String>("table_name"),
    //                     columns: Vec::new(),
    //                 };
    //                 Self::get_row_postgres_columns_for_table(res_row, &mut new_table);
    //                 schema_info.push(new_table);
    //             }
    //         };
    //     }
    //     Self::generate_mapped_table_entities(schema_info)
    // }

    // /// Groups all the [`RowTable`] entities that contains the info about a complete table into
    // /// a single entity of type [`DatabaseTable`]
    // fn generate_mapped_table_entities<'b>(schema_info: Vec<RowTable>) -> Vec<DatabaseTable<'b>> {
    //     let mut database_tables = Vec::new();

    //     for mapped_table in &schema_info {
    //         let unique_database_table = database_tables
    //             .iter_mut()
    //             .find(|table: &&mut DatabaseTable| table.table_name == mapped_table.table_name);
    //         match unique_database_table {
    //             Some(table) => Self::map_splitted_column_info_into_entity(mapped_table, table),
    //             None => {
    //                 let mut new_unique_database_table = DatabaseTable {
    //                     table_name: mapped_table.table_name.clone().to_string(),
    //                     columns: Vec::new(),
    //                 };
    //                 Self::map_splitted_column_info_into_entity(
    //                     mapped_table,
    //                     &mut new_unique_database_table,
    //                 );
    //                 database_tables.push(new_unique_database_table);
    //             }
    //         };
    //     }

    //     database_tables
    // }

    // /// Generates the [`DatabaseTableColumn`] elements that represents the metadata and information of a table column
    // /// and belongs to a concrete [`DatabaseTable`], being them extracted from a [`RowTable`] element that
    // /// it's related to only one table
    // fn map_splitted_column_info_into_entity(
    //     mapped_table: &RowTable,
    //     table_entity: &mut DatabaseTable,
    // ) {
    //     let mut entity_column = DatabaseTableColumn::new();
    //     for (idx, column) in mapped_table.columns.iter().enumerate() {
    //         let column_identifier = &column.column_identifier;
    //         if column_identifier == "column_name" {
    //             if let ColumnTypeValue::StringValue(value) = &column.value {
    //                 entity_column.column_name = value.to_owned().unwrap()
    //             }
    //         } else if column_identifier == "data_type" {
    //             if let ColumnTypeValue::StringValue(value) = &column.value {
    //                 entity_column.postgres_datatype = value.to_owned().unwrap()
    //             }
    //         } else if column_identifier == "character_maximum_length" {
    //             if let ColumnTypeValue::IntValue(value) = &column.value {
    //                 entity_column.character_maximum_length = value.to_owned()
    //             }
    //         } else if column_identifier == "is_nullable" {
    //             if let ColumnTypeValue::StringValue(value) = &column.value {
    //                 entity_column.is_nullable = matches!(value.as_ref().unwrap().as_str(), "YES")
    //             }
    //         } else if column_identifier == "column_default" {
    //             if let ColumnTypeValue::StringValue(value) = &column.value {
    //                 entity_column.column_default = value.to_owned()
    //             }
    //         } else if column_identifier == "numeric_precision" {
    //             if let ColumnTypeValue::IntValue(value) = &column.value {
    //                 entity_column.numeric_precision = value.to_owned()
    //             }
    //         } else if column_identifier == "numeric_scale" {
    //             if let ColumnTypeValue::IntValue(value) = &column.value {
    //                 entity_column.numeric_scale = value.to_owned()
    //             }
    //         } else if column_identifier == "numeric_precision_radix" {
    //             if let ColumnTypeValue::IntValue(value) = &column.value {
    //                 entity_column.numeric_precision_radix = value.to_owned()
    //             }
    //         } else if column_identifier == "datetime_precision" {
    //             if let ColumnTypeValue::IntValue(value) = &column.value {
    //                 entity_column.datetime_precision = value.to_owned()
    //             }
    //         } else if column_identifier == "interval_type" {
    //             if let ColumnTypeValue::StringValue(value) = &column.value {
    //                 entity_column.interval_type = value.to_owned()
    //             }
    //         } else if column_identifier == "foreign_key_info" {
    //             if let ColumnTypeValue::StringValue(value) = &column.value {
    //                 entity_column.foreign_key_info = value.to_owned()
    //             }
    //         } else if column_identifier == "foreign_key_name" {
    //             if let ColumnTypeValue::StringValue(value) = &column.value {
    //                 entity_column.foreign_key_name = value.to_owned()
    //             }
    //         } else if column_identifier == "primary_key_info" {
    //             if let ColumnTypeValue::StringValue(value) = &column.value {
    //                 entity_column.primary_key_info = value.to_owned()
    //             }
    //         } else if column_identifier == "primary_key_name" {
    //             if let ColumnTypeValue::StringValue(value) = &column.value {
    //                 entity_column.primary_key_name = value.to_owned()
    //             }
    //         } else if column_identifier == "is_identity" {
    //             if let ColumnTypeValue::StringValue(value) = &column.value {
    //                 entity_column.is_identity = matches!(value.as_ref().unwrap().as_str(), "YES")
    //             }
    //         } else if column_identifier == "identity_generation" {
    //             if let ColumnTypeValue::StringValue(value) = &column.value {
    //                 entity_column.identity_generation = value.to_owned()
    //             }
    //         };
    //         // Just for split the related column data into what will be the values for
    //         // every DatabaseTableColumn.
    //         // Every times that we find an &RelatedColumn which column identifier
    //         // is == "identity_generation", we know that we finished to set the values
    //         // for a new DatabaseTableColumn
    //         if &column.column_identifier == "identity_generation" {
    //             table_entity.columns.push(entity_column.clone());
    //             if idx == mapped_table.columns.len() - 1 {
    //                 entity_column = DatabaseTableColumn::new();
    //             }
    //         }
    //     }
    // }

    // /// Maps a [`tokio_postgres`] [`Row`] from the `information_schema` table into a `Canyon's` [`RowTable`],
    // /// by extracting every single column in a Row and making a relation between the column's name,
    // /// the datatype of the value that it's holding, and the value itself.
    // fn get_row_postgres_columns_for_table(res_row: &tokio_postgres::Row, table: &mut RowTable) {
    //     // for postgre_column in res_row.columns().iter() {
    //     //     if postgre_column.name() != "table_name" {
    //     //         // Discards the column "table_name"
    //     //         let mut new_column = RelatedColumn {
    //     //             column_identifier: postgre_column.name().to_string(),
    //     //             // datatype: postgre_column.type_().to_string(),
    //     //             value: ColumnTypeValue::NoneValue,
                    
    //     //         };

    //     //         match *postgre_column.type_() {
    //     //             Type::NAME | Type::VARCHAR | Type::TEXT => {
    //     //                 new_column.value = ColumnTypeValue::StringValue(
    //     //                     res_row.get::<&str, Option<String>>(postgre_column.name()),
    //     //                 );
    //     //             }
    //     //             Type::INT4 => {
    //     //                 new_column.value = ColumnTypeValue::IntValue(
    //     //                     res_row.get::<&str, Option<i32>>(postgre_column.name()),
    //     //                 );
    //     //             }
    //     //             _ => new_column.value = ColumnTypeValue::NoneValue,
    //     //         }
    //     //         table.columns.push(new_column)
    //     //     }
    //     // }
    // }
}
