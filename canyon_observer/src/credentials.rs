///! UNIMPLEMENTED
/// This crate will replace the action of retrieve the database credentials
/// in order to only wire the once time to the entire program's lifetime

use std::{fs, collections::HashMap};

/// Manages to retrieve the credentials to the desired database connection from an
/// handcoded `Secrets.toml` file, located at the root of the project.
#[derive(Clone, Debug)]
pub struct DatabaseCredentials {
    pub username: String,
    pub password: String,
    pub db_name: String,
}

impl DatabaseCredentials{
    
    pub fn new() -> Self {

        let parsed_credentials = DatabaseCredentials::credentials_parser();

        Self {
            username: parsed_credentials.get("username").unwrap().to_owned(),
            password: parsed_credentials.get("password").unwrap().to_owned(),
            db_name: parsed_credentials.get("db_name").unwrap().to_owned()
        }
    }

    pub fn credentials_parser() -> HashMap<String, String> {
    
        const FILE_NAME: &str = "Secrets.toml";
        let mut credentials_mapper: HashMap<_, _> = HashMap::new();
        
        let secrets_file = fs::read_to_string(FILE_NAME)
            .expect( // TODO Convert this to a custom error
                &(format!(
                    "\n\nNo file --> {} <-- founded on the root of this project.", FILE_NAME
                ) + "\nPlease, ensure that you created one .toml file with the necesary"
                + " properties needed in order to connect to the database.\n\n")
            );

        let secrets_file_splitted = secrets_file
            .split_terminator("\n");

        for entry in secrets_file_splitted {
            let cleaned_entry = 
                entry
                .split_ascii_whitespace()
                .filter(
                    |x| x != &"="
                );

            let mut pair = Vec::new();
            cleaned_entry.for_each(
                |elem| pair.push(elem.to_string())
            );

            let attr = pair.get(0).unwrap();
            let value = pair.get(1).unwrap();
            
            credentials_mapper.insert(attr.to_owned(), value.to_owned());
        }
        
        credentials_mapper
    }
}