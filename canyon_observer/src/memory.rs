use std::collections::HashMap;
use walkdir::WalkDir;
use std::fs;
use canyon_crud::crud::Transaction;

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
    pub memory: HashMap<String, String>
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
        .wrapper;

        for row in mem_results {
            println!();
            println!("Mem Row: {:?}", row)
        }
        

        let mut mem = Self {
            memory: HashMap::new()
        };

        for file in WalkDir::new("./src").into_iter().filter_map(|file| file.ok()) {
            if file.metadata().unwrap().is_file() 
                && file.path().display().to_string().ends_with(".rs") 
            {
                // println!("{} -> RS source file", file.path().display());
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
                        // println!("Line:{}", line);
                        canyon_entity_macro_counter += 1;
                    }
                }

                // If more than two, we panic!
                if canyon_entity_macro_counter > 1 {
                    // compile_error!(...)
                } else if canyon_entity_macro_counter == 1 {
                    mem.memory.insert(
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

        println!("MEM: {:?}", &mem);

        // Insert into the memory table the new discovered entities
        // Care, insert the new ones, delete the old
        let mut values_to_insert = String::new();
        for (filename, struct_name) in &mem.memory {
            values_to_insert.push_str(
                format!("('{}', '{}'),", filename, struct_name).as_str()
            );
        }
        // Replace the last ',' for a ';'
        values_to_insert.pop();
        values_to_insert.push_str(";");
        println!("Values to insert: \n{:?}", &values_to_insert);

        let insert_memory_result = Self::query(
            format!(
                "INSERT INTO canyon_memory (filename, struct_name) VALUES {}", 
                values_to_insert
            ).as_str(), 
            &[]
        ).await
        .wrapper;

        println!("Insert memory result: {:?}", &insert_memory_result);

        mem
    }

    /// Generates, if not exists the `canyon_memory` table
    async fn create_memory() {
        Self::query(
            "CREATE TABLE IF NOT EXISTS canyon_memory \
            ( id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY, \
              filename VARCHAR NOT NULL, struct_name VARCHAR NOT NULL
            )", 
            &[]
        ).await.wrapper;
    }
}