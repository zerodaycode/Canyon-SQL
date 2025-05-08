use crate::canyon::Canyon;
use crate::connection::contracts::DbConnection;
use crate::mapper::RowMapper;
use crate::query::parameters::QueryParameter;
use crate::transaction::Transaction;
use std::error::Error;
use std::fmt::Debug;
use std::ops::DerefMut;
// TODO: all the query works here
// TODO: exports things like Select::... where receives the table
// name and prepares the raw query (maybe with const_format!) for improved performance

// TODO: query should implement ToStatement (as the drivers underneath Canyon) or similar
// to be usable directly in the input of Transaction and DbConnenction
/// Holds a sql sentence details
///
/// Plan: The MacroTokens struct gets some generic bounds to retrieve the fields names at compile
/// time (already does it) and the querybuilder uses it with const_format to introduce the names of the
/// columns instead of just using * (in this case, is the same, unless we introduce new annotations like #[skip_mapping]
#[derive(Debug)]
pub struct Query<'a> {
    pub sql: String,
    pub params: Vec<&'a dyn QueryParameter<'a>>,
}

impl<'a> Query<'a> {
    pub fn new(sql: String) -> Query<'a> {
        Self {
            sql,
            params: vec![],
        }
    }

    /// Launches the generated query against the database assuming the default
    /// [`DbConnection`]
    pub async fn launch_default<T: Transaction + RowMapper>(
        self,
    ) -> Result<Vec<T>, Box<(dyn Error + Send + Sync + 'a)>>
    where
        Vec<T>: FromIterator<<T as RowMapper>::Output>,
    {
        let mut input = Canyon::instance()?.get_default_connection().await?;
        <T as Transaction>::query(&self.sql, &self.params, input.deref_mut()).await
    }

    /// Launches the generated query against the database with the selected [`DbConnection`]
    pub async fn launch_with<I: DbConnection, R: RowMapper>(
        self,
        input: I,
    ) -> Result<Vec<R>, Box<(dyn Error + Send + Sync + 'a)>>
    where
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        input.query(&self.sql, &self.params).await
    }
}

