use tokio_postgres::{Client, Connection, Error, NoTls, Socket, tls::NoTlsStream};
use std::{fs, collections::HashMap, marker::PhantomData};

/// Manages to retrieve the credentials to the desired database connection from an
/// handcoded `secrets.toml` file, located at the root of the project.
pub struct DatabaseCredentials {
    username: String,
    password: String,
    host: String,
    db_name: String
}

impl DatabaseCredentials{

    pub fn new() -> Self {

        let parsed_credentials = DatabaseCredentials::credentials_parser();

        Self {
            username: parsed_credentials.get("username").unwrap().to_owned(),
            password: parsed_credentials.get("password").unwrap().to_owned(),
            host:parsed_credentials.get("host").unwrap().to_owned(),
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

/// Creates a new connection with a database, returning the necessary tools
/// to query the created link.
/// TODO: Explain how to use this struct independently from CRUD trait
pub struct DatabaseConnection<'a> {
    pub client: Client,
    pub connection: Connection<Socket, NoTlsStream>,
    pub phantom: &'a PhantomData<DatabaseConnection<'a>>
}

unsafe impl Send for DatabaseConnection<'_> {}
unsafe impl Sync for DatabaseConnection<'_> {}

impl<'a> DatabaseConnection<'a> {

    pub async fn new() -> Result<DatabaseConnection<'a>, Error> {

        let credentials = DatabaseCredentials::new();

        let (new_client, new_connection) =
            tokio_postgres::connect(
            &format!(
                "postgres://{user}:{pswd}@{host}/{db}",
                    user = credentials.username,
                    pswd = credentials.password,
                    host = credentials.host,
                    db = credentials.db_name
                )[..], 
            NoTls)
            .await?;

        Ok(Self {
            client: new_client,
            connection: new_connection,
            phantom: &PhantomData
        })
    }
}
