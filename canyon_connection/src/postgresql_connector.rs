use std::marker::PhantomData;
use tokio_postgres::{Client, Connection, Error, NoTls, Socket, tls::NoTlsStream};

use crate::credentials::DatabaseCredentials;


/// Represents a connection with a `PostgreSQL` database
pub struct DatabaseConnection<'a> {
    pub client: Client,
    pub connection: Connection<Socket, NoTlsStream>,
    pub phantom: &'a PhantomData<DatabaseConnection<'a>>
}

unsafe impl Send for DatabaseConnection<'_> {}
unsafe impl Sync for DatabaseConnection<'_> {}

impl<'a> DatabaseConnection<'a> {

    pub async fn new(credentials: &DatabaseCredentials) -> Result<DatabaseConnection<'a>, Error> {
        let (new_client, new_connection) =
            tokio_postgres::connect(
            &format!(
                "postgres://{user}:{pswd}@{host}/{db}",
                    user = credentials.username,
                    pswd = credentials.password,
                    host = credentials.host,
                    db = credentials.db_name
                )[..], 
            NoTls
            ).await?;

        Ok(Self {
            client: new_client,
            connection: new_connection,
            phantom: &PhantomData
        })
    }
}


