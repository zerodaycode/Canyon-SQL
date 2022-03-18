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

        let mut schema_info: Vec<Table> = Vec::new();

        for (i, res_row) in results.iter().enumerate() {
            
            // let schema_info_row = DatabaseDataRows {
            //     table_name: res_row.get::<&str, String>("table_name"),
            //     columns_types: HashMap::new()
            // };
            let mut table = Table { 
                table_name: res_row.get::<&str, String>("table_name"),
                columns: Vec::new(),
            };

            println!("\nRow INDEX: {:?}", i);
            for (i, column) in res_row.columns().iter().enumerate() {

                let mut new_column = Column {
                    column_name: column.name().to_string(), 
                    datatype: column.type_().to_string(), 
                    value: ColumnTypeValue::NoneValue
                };
                
                println!("Column Index: {}; Column name: {:?}, Column type: {:?}", i, column.name(), column.type_());
                match *column.type_() {
                    Type::NAME | Type::VARCHAR => {
                        let str_value = res_row.get::<&str, Option<String>>(column.name());
                        println!("Value: {:?}", str_value);
                        new_column.value = ColumnTypeValue::ValorString(str_value);
                    },
                    Type::INT4 => {
                        let int_value = res_row.get::<&str, Option<i32>>(column.name());
                        println!("Value: {:?}", int_value);
                        new_column.value = ColumnTypeValue::ValorInt(int_value);
                    },
                    _ => new_column.value = ColumnTypeValue::NoneValue
                }
                table.columns.push(new_column)
            }
            schema_info.append(&mut vec![table]);
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
    pub column_name: String,
    pub datatype: String,
    pub value: ColumnTypeValue
}

#[derive(Debug)]
pub enum ColumnTypeValue {
    ValorString(Option<String>),
    ValorInt(Option<i32>),
    NoneValue
}