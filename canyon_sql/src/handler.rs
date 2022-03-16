/// Provides the necessary entities to let Canyon perform and develop
/// it's full potential, completly managing all the entities written by
/// the user and annotated with the `#[canyon_entity]`

use std::collections::HashMap;
use tokio_postgres::types::Type;

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
    pub canyon_tables:Vec<Table>,
    pub database_tables:Vec<Table>
}
// Makes this structure able to make queries to the database
impl Transaction<Self> for CanyonHandler {}

impl CanyonHandler {
    // pub fn new() -> Self{
        
    // }
    pub async fn fetch_database_status() {
        let query_request = 
            "SELECT table_name, column_name, data_type, 
                character_maximum_length,is_nullable,column_default,numeric_precision,
                numeric_scale,numeric_precision_radix,datetime_precision,interval_type 
            FROM information_schema.columns WHERE table_schema = 'public';";
        
        let results = Self::query(query_request, &[]).await.wrapper;

        let mut schema_info: Vec<DatabaseDataRows> = Vec::new();
        
        for (i, res_row) in results.iter().enumerate() {
            
            let schema_info_row = DatabaseDataRows {
                table_name: res_row.get::<&str, String>("table_name"),
                columns_types: HashMap::new()
            };

            println!("\nRow INDEX: {:?}", i);
            let name = res_row.columns()[i].name();
            let type_ = res_row.columns()[i].type_();
            for (i, column) in res_row.columns().iter().enumerate() {
                
                println!("Column Index: {}; Column name: {:?}, Column type: {:?}", i, column.name(), column.type_());
                match *column.type_() {
                    Type::NAME => {
                        println!("Value: {:?}", res_row.get::<&str, Option<String>>(column.name()));
                    },
                    Type::VARCHAR => {
                        println!("Value: {:?}", res_row.get::<&str, Option<String>>(column.name()));
                    },
                    Type::INT4 => {
                        println!("Value: {:?}", res_row.get::<&str, Option<i32>>(column.name()));
                    },
                    _ => println!("No, primo")
                }
            }
            println!("\nTable: {:?}", schema_info_row);
            schema_info.append(&mut vec![schema_info_row]);
        }
        println!("\nCollection: {:?}", schema_info);
    }
}

#[derive(Debug)]
pub struct Table {
    pub table_name: String,
    pub columns: Vec<Column>
} 
// impl Table {
//     pub fn new() -> Self{

//     }
// }

#[derive(Debug)]
pub struct  Column {
    pub name: String,
    pub datatype: String
}