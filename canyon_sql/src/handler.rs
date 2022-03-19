/// Provides the necessary entities to let Canyon perform and develop
/// it's full potential, completly managing all the entities written by
/// the user and annotated with the `#[canyon_entity]`

use std::collections::HashMap;
use tokio_postgres::{types::Type, Row};

use crate::crud::Transaction;

/// A struct to manages the retrieved rows with the schema info for the
/// current user's selected database
#[derive(Debug, Clone)]
pub struct DatabaseDataRows {
    pub table_name: String,
    pub columns_types: HashMap<String, String>
}

/// TODO This should be the Canyon register after refactor the mut static 
/// variable on the observer on some complex and convenient data structure
/// to pass the data retrieved on compile time to runtime
#[derive(Debug)]
pub struct CanyonHandler {
    pub canyon_tables:Vec<DatabaseTable>,
    // pub database_tables:Vec<CanyonEntity>  // Canyony
}
// Makes this structure able to make queries to the database
impl Transaction<Self> for CanyonHandler {}

impl CanyonHandler {
    pub async fn fetch_database_status() {
        let query_request = 
            "SELECT table_name, column_name, data_type, 
                character_maximum_length,is_nullable,column_default,numeric_precision,
                numeric_scale,numeric_precision_radix,datetime_precision,interval_type 
            FROM information_schema.columns WHERE table_schema = 'public';";
        
        let results = Self::query(query_request, &[]).await.wrapper;

        let mut schema_info: Vec<RowTable> = Vec::new();

        for res_row in results.iter() {
            // println!("\nres_row: {:?}", &res_row);
            // If already exists a row that belongs to a table, add the founded columns
            match schema_info.iter_mut().find( |table| {
                table.table_name == res_row.get::<&str, String>("table_name")
            }) {
                unique_table => {
                    match unique_table {
                        Some(table) => {
                            Self::get_row_postgre_columns_for_table(&res_row, table);
                        }
                        None => {
                            let mut new_table = RowTable { 
                                table_name: res_row.get::<&str, String>("table_name"),
                                columns: Vec::new(),
                            };
                            Self::get_row_postgre_columns_for_table(&res_row, &mut new_table);
                            schema_info.push(new_table);
                        }
                    }
                }
            }
        }
        println!("FINISHED TABLES TOTAL => {}", &schema_info.len());
        for table in &schema_info {
            println!("\nFinal table: {:?}", table.table_name);
            for column in &table.columns {
                println!("Column: {:?}, Type: {:?}, Value: {:?}", 
                    &column.column_name, &column.datatype, &column.value
                );
            }
        }
    }

    /// Retrieves for every row founded related to one table record, 
    /// the data and values associated to that row.
    /// So, for every row, here we have rows containing values related to one table, but
    /// are columns that, at the end of the function, just represents the data stored in one
    /// row of results.
    pub fn get_row_postgre_columns_for_table(res_row: &Row, table: &mut RowTable) {
        for postgre_column in res_row.columns().iter() {

            if postgre_column.name().to_string() != "table_name" { // Discards the column "table_name"
                let mut new_column = RelatedColumn {
                    column_name: postgre_column.name().to_string(), 
                    datatype: postgre_column.type_().to_string(), 
                    value: ColumnTypeValue::NoneValue
                };
    
                match *postgre_column.type_() {
                    Type::NAME | Type::VARCHAR => {
                        new_column.value = ColumnTypeValue::StringValue(
                            res_row.get::<&str, Option<String>>(postgre_column.name())
                        );
                    },
                    Type::INT4 => {
                        new_column.value = ColumnTypeValue::IntValue(    
                            res_row.get::<&str, Option<i32>>(postgre_column.name())
                        );
                    },
                    _ => new_column.value = ColumnTypeValue::NoneValue
                }
                table.columns.push(new_column)
            }
        }
    }
}


/* Models that represents the database entities that belongs to the current schema */
#[derive(Debug)]
pub struct DatabaseTable {
    pub table_name: String,
    pub columns: Vec<DatabaseTableColumn>
}
#[derive(Debug)]
pub struct DatabaseTableColumn {
    pub column_name: String,
    pub postgres_datatype: String,
    pub character_maximum_length: i32,
    pub is_nullable: bool,  // Care, postgres type is varchar
    pub column_default: String,
    pub numeric_precision: i32,
    pub numeric_precision_radix: i32,
    pub datetime_precision: i32,
    pub interval_type: String
}


/* POSTGRESQL entities for map the multiple rows that are related to one table, and the multiple
    columns that are nested related to those row table */

#[derive(Debug)]
pub struct RowTable {
    pub table_name: String,
    pub columns: Vec<RelatedColumn>
} 
#[derive(Debug)]
pub struct RelatedColumn {
    pub column_name: String,
    pub datatype: String,
    pub value: ColumnTypeValue
}

#[derive(Debug)]
pub enum ColumnTypeValue {
    StringValue(Option<String>),
    IntValue(Option<i32>),
    NoneValue
}