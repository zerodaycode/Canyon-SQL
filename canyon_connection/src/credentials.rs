use std::{fs, collections::HashMap};
use serde::Deserialize;

/// ```
#[test]
fn load_ds_config_from_array() {
    const CONFIG_FILE_MOCK_ALT: &'static str = r#"
        [canyon_sql]
        datasources = [
            {name = 'PostgresDS', properties.db_type = 'postgresql', properties.username = 'username', properties.password = 'random_pass', properties.host = 'localhost', properties.db_name = 'triforce'},
            {name = 'SqlServerDS', properties.db_type = 'sqlserver', properties.username = 'username2', properties.password = 'random_pass2', properties.host = '192.168.0.250.1:3340', properties.db_name = 'triforce2'}
        ]
    "#;

    let config: CanyonSqlConfig = toml::from_str(CONFIG_FILE_MOCK_ALT)
        .expect("A failure happened retrieving the [canyon_sql] section");

        let ds_0 = &config.canyon_sql.datasources[0];
        let ds_1 = &config.canyon_sql.datasources[1];
            
        assert_eq!(ds_0.name, "PostgresDS");
        assert_eq!(ds_0.properties.db_type, "postgresql");
        assert_eq!(ds_0.properties.username, "username");
        assert_eq!(ds_0.properties.password, "random_pass");
        assert_eq!(ds_0.properties.host, "localhost");
        assert_eq!(ds_0.properties.db_name, "triforce");

        assert_eq!(ds_1.name, "SqlServerDS");
        assert_eq!(ds_1.properties.db_type, "sqlserver");
        assert_eq!(ds_1.properties.username, "username2");
        assert_eq!(ds_1.properties.password, "random_pass2");
        assert_eq!(ds_1.properties.host, "192.168.0.250.1:3340");
        assert_eq!(ds_1.properties.db_name, "triforce2");
}

/// ```
#[derive(Deserialize, Debug)]
pub struct CanyonSqlConfig<'a> {
    #[serde(borrow)]
    pub canyon_sql: Datasources<'a>
}
#[derive(Deserialize, Debug)]
pub struct Datasources<'a> {
    #[serde(borrow)]
    pub datasources: Vec<Datasource<'a>>
}

#[derive(Deserialize, Debug)]
pub struct Datasource<'a> {
    #[serde(borrow)]
    pub name: &'a str, 
    pub properties: DatasourceProperties<'a>
} 

#[derive(Deserialize, Debug)]
pub struct DatasourceProperties<'a> {
    pub db_type: &'a str,  
    pub username: &'a str, 
    pub password: &'a str,
    pub host: &'a str,
    pub db_name: &'a str,
}


/// Manages to retrieve the credentials to the desired database connection from an
/// handcoded `secrets.toml` file, located at the root of the project.
pub struct DatabaseCredentials {
    pub db_type: DatabaseType,
    pub username: String,
    pub password: String,
    pub host: String,
    pub db_name: String
}

impl DatabaseCredentials {
    pub fn new() -> Self {
        let parsed_credentials = DatabaseCredentials::credentials_parser();

        Self {
            db_type: DatabaseType::PostgreSql,
            username: parsed_credentials.get("username").unwrap().to_owned(),
            password: parsed_credentials.get("password").unwrap().to_owned(),
            host:parsed_credentials.get("host").unwrap_or(&"localhost".to_string()).to_owned(),
            db_name: parsed_credentials.get("db_name").unwrap().to_owned()
        }
    }

    pub fn credentials_parser() -> HashMap<String, String> {
        const FILE_NAME: &str = "secrets.toml";
        let mut credentials_mapper: HashMap<_, _> = HashMap::new();

        let secrets_file = fs::read_to_string(FILE_NAME)
            .expect( // TODO Convert this to a custom error
                &(format!(
                    "\n\nNo file --> {} <-- founded on the root of this project.", FILE_NAME
                ) + "\nPlease, ensure that you created one .toml file with the necesary"
                + " properties needed in order to connect to the database.\n\n")
            );

        let secrets_file_splitted = secrets_file
            .split_terminator('\n');

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


pub enum DatabaseType {
    PostgreSql,
    SqlServer
}