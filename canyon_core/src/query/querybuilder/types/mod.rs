pub mod delete;
pub mod select;
pub mod update;

pub use self::{delete::*, select::*, update::*};
use crate::connection::database_type::DatabaseType;
use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Operator;
use crate::query::parameters::QueryParameter;
use crate::query::query::Query;
use std::error::Error;

/// Type for construct more complex queries than the classical CRUD ones.
pub struct QueryBuilder<'a> {
    pub(crate) sql: String,
    pub(crate) params: Vec<&'a dyn QueryParameter<'a>>,
    pub(crate) database_type: DatabaseType,
}

unsafe impl Send for QueryBuilder<'_> {}
unsafe impl Sync for QueryBuilder<'_> {}

impl<'a> QueryBuilder<'a> {
    pub fn new(
        sql: String,
        database_type: DatabaseType,
    ) -> Result<Self, Box<(dyn Error + Send + Sync + 'a)>> {
        Ok(Self {
            sql,
            params: vec![], // TODO: as option? and then match it for emptyness and pass &[] if possible?
            database_type,
        })
    }

    pub fn build(mut self) -> Result<Query<'a>, Box<(dyn Error + Send + Sync)>> {
        // TODO: here we should check for our invariants
        self.sql.push(';');
        Ok(Query {
            sql: self.sql,
            params: self.params,
        })
    }

    pub fn r#where<Z: FieldValueIdentifier<'a>>(&mut self, r#where: Z, op: impl Operator) {
        let (column_name, value) = r#where.value();

        let where_ = String::from(" WHERE ")
            + column_name
            + &op.as_str(self.params.len() + 1, &self.database_type);

        self.sql.push_str(&where_);
        self.params.push(value);
    }

    pub fn and<Z: FieldValueIdentifier<'a>>(&mut self, r#and: Z, op: impl Operator) {
        let (column_name, value) = r#and.value();

        let and_ = String::from(" AND ")
            + column_name
            + &op.as_str(self.params.len() + 1, &self.database_type);

        self.sql.push_str(&and_);
        self.params.push(value);
    }

    pub fn and_values_in<Z, Q>(&mut self, r#and: Z, values: &'a [Q])
    where
        Z: FieldIdentifier,
        Q: QueryParameter<'a>,
    {
        if values.is_empty() {
            return;
        }

        self.sql.push_str(&format!(" AND {} IN (", r#and.as_str()));

        let mut counter = 1;
        values.iter().for_each(|qp| {
            if values.len() != counter {
                self.sql.push_str(&format!("${}, ", self.params.len()));
                counter += 1;
            } else {
                self.sql.push_str(&format!("${}", self.params.len()));
            }
            self.params.push(qp)
        });

        self.sql.push(')');
    }

    pub fn or_values_in<Z, Q>(&mut self, r#or: Z, values: &'a [Q])
    where
        Z: FieldIdentifier,
        Q: QueryParameter<'a>,
    {
        if values.is_empty() {
            return;
        }

        self.sql.push_str(&format!(" OR {} IN (", r#or.as_str()));

        let mut counter = 1;
        values.iter().for_each(|qp| {
            if values.len() != counter {
                self.sql.push_str(&format!("${}, ", self.params.len()));
                counter += 1;
            } else {
                self.sql.push_str(&format!("${}", self.params.len()));
            }
            self.params.push(qp)
        });

        self.sql.push(')');
    }

    pub fn or<Z: FieldValueIdentifier<'a>>(&mut self, r#and: Z, op: impl Operator) {
        let (column_name, value) = r#and.value();

        let and_ = String::from(" OR ")
            + column_name
            + &op.as_str(self.params.len() + 1, &self.database_type);

        self.sql.push_str(&and_);
        self.params.push(value);
    }

    #[inline]
    pub fn order_by<Z: FieldIdentifier>(&mut self, order_by: Z, desc: bool) {
        self.sql.push_str(
            &(format!(
                " ORDER BY {}{}",
                order_by.as_str(),
                if desc { " DESC " } else { "" }
            )),
        );
    }
}
