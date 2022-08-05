use std::collections::HashMap;
use walkdir::WalkDir;
use std::fs;
use canyon_crud::crud::Transaction;

use crate::QUERIES_TO_EXECUTE;

/// Convenient struct that contains the necessary data and operations to implement
/// the `Canyon Memory`.
/// 
/// Canyon Memory it's just a convenient way of relate the data of a Rust source
/// code file and the `CanyonEntity` (if so), helping Canyon to know what source
/// file contains a `#[canyon_entity]` annotation and restricting it to just one
/// annotated struct per file.
/// 
/// This limitation it's imposed by desing. Canyon, when manages all the entities in
/// the user's source code, needs to know for future migrations the old data about a structure
/// and the new modified one.
/// 
/// For example, let's say that you have a:
/// ```
/// pub struct Person {
///    /* some fields */ 
/// }
/// ```
/// 
/// and you decided to modify it's Ident and change it to `Human`.
/// 
/// Canyon will take care about modifying the Database, and `ALTER TABLE` to edit the actual data for you,
/// but, if it's not able to get the data to know that the old one is `Person` and the new one it's `Human`.
/// it will simply drop the table (losing all your data) and creating a new table `Human`.
/// 
/// So, we decised to follow the next approach:
/// Every entity annotated with a `#[canyon_entity]` annotation will be related to only unique Rust source
/// code file. If we find more, Canyon will raise and error saying that it does not allows to having more than
/// one managed entity per source file.
/// 
/// Then, we will store the entities data in a special table only for Canyon, where we will create the relation 
/// between the source file, the entity and it's fields and data.
/// 
/// So, if the user wants or needs to modify the data of it's entity, Canyon can secure that will perform the
/// correct operations because we can't "remember" how that entity was, and how it should be now, avoiding
/// potencially dangerous operations due to lack of knowing what entity relates with new data.
/// 
/// The `memory field` HashMap is made by the filename as a key, and the struct's name as value
#[derive(Debug)]
pub struct CanyonMemory {
    pub memory: HashMap<String, String>,
    pub table_rename: HashMap<String, String>
}

// Makes this structure able to make queries to the database
impl Transaction<Self> for CanyonMemory {}

impl CanyonMemory {
    pub async fn remember() -> Self {

        // Creates the memory table if not exists
        Self::create_memory().await;
        // Check database for the "memory data"
        let mem_results = Self::query(
            "SELECT * FROM canyon_memory",
            &[]
        ).await
        .ok()
        .expect("Error querying Canyon Memory")
        .wrapper;

        // Manually maps the results
        let mut memory_db_rows = Vec::new();
        // let mut memory_rows_to_delete = Vec::new();
        // Cando non a encontres no parseo de archivos, acumulas no array
        // Tremendísima query con WHERE IN (45)
        for row in mem_results {
            let db_row =  CanyonMemoryDatabaseRow {
                id: row.get::<&str, i32>("id"),
                filename: row.get::<&str, String>("filename"),
                struct_name: row.get::<&str, String>("struct_name"),
            };
            memory_db_rows.push(db_row);
        }
        

        // Parses the source code files looking for the #[canyon_entity] annotated classes
        let mut mem = Self {
            memory: HashMap::new(),
            table_rename: HashMap::new(),
        };
        Self::find_canyon_entity_annotated_structs(&mut mem).await;


        // Insert into the memory table the new discovered entities
        // Care, insert the new ones, delete the olds
        // Also, updates the registry when the fields changes
        let mut values_to_insert = String::new();
        let mut updates = Vec::new();

        for (filename, struct_name) in &mem.memory {
            // When the filename and the struct hasn't been modified and are already on db
            let already_in_db = memory_db_rows
                .iter()
                .any( |el| 
                    {
                        (el.filename == *filename && el.struct_name == *struct_name) ||
                        (
                            (el.filename != *filename && el.struct_name == *struct_name) ||
                            (el.filename == *filename && el.struct_name != *struct_name)
                        )
                    }
                );
            if !already_in_db {
                values_to_insert.push_str(
                    format!("('{}', '{}'),", filename, struct_name).as_str()
                );
            }
            // When the struct or the filename it's already on db but one of the two has been modified
            let need_to_update = memory_db_rows
                .iter()
                .filter( |el| 
                    {
                        (el.filename == *filename || el.struct_name == *struct_name) &&
                        !(el.filename == *filename && el.struct_name == *struct_name)
                    }
                ).next();

            if let Some(update) = need_to_update {
                updates.push(struct_name);
                QUERIES_TO_EXECUTE.lock().unwrap().push(
                    format!(
                        "UPDATE canyon_memory SET filename = '{}', struct_name = '{}'\
                            WHERE id = {}", 
                        filename, struct_name, update.id
                    )
                );

                // if the updated element is the struct name, whe add it to the table_rename Hashmap
                let rename_table = &update.struct_name != struct_name;

                if rename_table {
                    println!("Adding a new table to rename. new name: {}, old name {}", struct_name.clone(), update.struct_name.clone());
                    mem.table_rename.insert( struct_name.clone().to_lowercase(),update.struct_name.clone().to_lowercase());
                }
            }
        }


        if values_to_insert != String::new() {
            values_to_insert.pop();
            values_to_insert.push_str(";");
            
            QUERIES_TO_EXECUTE.lock().unwrap().push(
                format!(
                    "INSERT INTO canyon_memory (filename, struct_name) VALUES {}", 
                    values_to_insert
                )
            );
        }

        // Deletes the records when a table is dropped on the previous Canyon run
        let in_memory = mem.memory.values()
            .collect::<Vec<&String>>();
        memory_db_rows.into_iter()
            .for_each( |db_row|  
                {
                    if !in_memory.contains(&&db_row.struct_name) &&
                        !updates.contains(&&db_row.struct_name)
                    {
                        QUERIES_TO_EXECUTE.lock().unwrap().push(
                            format!(
                                "DELETE FROM canyon_memory WHERE struct_name = '{}'",
                                db_row.struct_name
                            )
                        );
                    }
                }
            );

        mem
    }

    /// Parses the Rust source code files to find the one who contains Canyon entities
    /// ie -> annotated with `#{canyon_entity}`
    async fn find_canyon_entity_annotated_structs(&mut self) {
        for file in WalkDir::new("./src").into_iter().filter_map(|file| file.ok()) {
            if file.metadata().unwrap().is_file() 
                && file.path().display().to_string().ends_with(".rs") 
            {
                // Opening the source code file
                let contents = fs::read_to_string(file.path())
                    .expect("Something went wrong reading the file");

                let mut canyon_entity_macro_counter = 0;
                let mut struct_name = String::new();
                for line in contents.split("\n") {
                    if line.starts_with("pub struct") {
                        struct_name.push_str(
                            line.split_whitespace()
                                .collect::<Vec<&str>>()
                                .get(2)
                                .unwrap_or(&"FAILED")
                        )
                    }
                    if line.contains("#[") && line.contains("canyon_entity]") 
                        && !line.starts_with("//") 
                    {
                        canyon_entity_macro_counter += 1;
                    }
                }

                // If more than two, we panic!
                if canyon_entity_macro_counter > 1 {
                    panic!(
                        r"Canyon does not support having multiple structs annotated\ 
                        with `#[canyon::entity]` on the same file when the `#[canyon]`\  
                        macro it's present on the program"
                    )
                } else if canyon_entity_macro_counter == 1 {
                    self.memory.insert(
                        file.path()
                            .display()
                            .to_string()
                            .replace("\\", "/")
                            .split("/")
                            .collect::<Vec<&str>>()
                            .last()
                            .unwrap_or(&"FAILED")
                            .to_string(),
                        struct_name
                    );
                }
            }
        }
    }

    /// Generates, if not exists the `canyon_memory` table
    async fn create_memory() {
        Self::query(
            "CREATE TABLE IF NOT EXISTS canyon_memory \
            ( id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY, \
              filename VARCHAR NOT NULL, struct_name VARCHAR NOT NULL
            )", 
            &[]
        ).await
        .ok()
        .expect("Error creating the 'canyon_memory' table")
        .wrapper;
    }
}


/// Represents a single row from the `canyon_memory` table
#[derive(Debug)]
struct CanyonMemoryDatabaseRow {
    id: i32,
    filename: String,
    struct_name: String
}