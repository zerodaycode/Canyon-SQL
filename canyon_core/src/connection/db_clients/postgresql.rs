use crate::{query::DbConnection, query_parameters::QueryParameter, rows::CanyonRows};

#[cfg(feature = "postgres")]
use tokio_postgres::Client;

/// A connection with a `PostgreSQL` database
#[cfg(feature = "postgres")]
pub struct PostgreSqlConnection {
    pub client: Client,
    // pub connection: Connection<Socket, NoTlsStream>, // TODO Hold it, or not to hold it... that's the question!
}

impl DbConnection for PostgreSqlConnection {
    fn launch<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl std::future::Future<
        Output = Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send)>>,
    > + Send {
        postgres_query_launcher::launch(stmt, params, self)
    }
}

#[cfg(feature = "postgres")]
pub(crate) mod postgres_query_launcher {
    use super::*;

    #[inline(always)]
    pub(crate) async fn launch<'a>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &PostgreSqlConnection,
    ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send)>> {
        let mut m_params = Vec::new();
        for param in params {
            m_params.push((*param).as_postgres_param());
        }

        let r = conn.client.query(stmt, m_params.as_slice()).await?;

        Ok(CanyonRows::Postgres(r))
    }
}
