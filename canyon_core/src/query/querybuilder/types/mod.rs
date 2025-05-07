pub mod delete;
pub mod select;
pub mod update;

pub use self::{delete::*, select::*, update::*};
use crate::connection::contracts::DbConnection;
use crate::connection::database_type::DatabaseType;
use crate::mapper::RowMapper;
use crate::query::parameters::QueryParameter;
use std::error::Error;
use std::marker::PhantomData;

/// Type for construct more complex queries than the classical CRUD ones.
pub struct QueryBuilder<'a, I: DbConnection + ?Sized, R: RowMapper> {
    // query: Query<'a>,
    pub(crate) sql: String,
    pub(crate) params: Vec<&'a dyn QueryParameter<'a>>,
    pub(crate) database_type: DatabaseType,
    pub(crate) input: &'a I,
    pd: PhantomData<R>,
}

unsafe impl<I: DbConnection + ?Sized, R: RowMapper> Send for QueryBuilder<'_, I, R> {}
unsafe impl<I: DbConnection + ?Sized, R: RowMapper> Sync for QueryBuilder<'_, I, R> {}

impl<'a, I: DbConnection + ?Sized, R: RowMapper> QueryBuilder<'a, I, R> {
    pub fn new(sql: String, input: &'a I) -> Result<Self, Box<(dyn Error + Send + Sync + 'a)>> {
        Ok(Self {
            sql,
            params: vec![],
            database_type: input.get_database_type()?,
            input,
            pd: Default::default(),
        })
    }

    /// Launches the generated query against the database targeted
    /// by the selected datasource
    pub async fn query(mut self) -> Result<Vec<R>, Box<(dyn Error + Send + Sync + 'a)>>
    where
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        self.sql.push(';');
        self.input.query(&self.sql, &self.params).await
    }
}
