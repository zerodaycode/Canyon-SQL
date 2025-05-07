use crate::connection::contracts::DbConnection;
use crate::mapper::RowMapper;
use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Operator;
use crate::query::parameters::QueryParameter;
pub(crate) use crate::query::querybuilder::QueryBuilder;

mod delete;
mod select;
mod update;

impl<'a, I: DbConnection + ?Sized, R: RowMapper> QueryBuilder<'a, I, R> {
    fn r#where<Z: FieldValueIdentifier<'a>>(&mut self, r#where: Z, op: impl Operator) {
        let (column_name, value) = r#where.value();

        let where_ = String::from(" WHERE ")
            + column_name
            + &op.as_str(self.params.len() + 1, &self.database_type);

        self.sql.push_str(&where_);
        self.params.push(value);
    }

    fn and<Z: FieldValueIdentifier<'a>>(&mut self, r#and: Z, op: impl Operator) {
        let (column_name, value) = r#and.value();

        let and_ = String::from(" AND ")
            + column_name
            + &op.as_str(self.params.len() + 1, &self.database_type);

        self.sql.push_str(&and_);
        self.params.push(value);
    }

    fn and_values_in<Z, Q>(&mut self, r#and: Z, values: &'a [Q])
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

    fn or_values_in<Z, Q>(&mut self, r#or: Z, values: &'a [Q])
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

    fn or<Z: FieldValueIdentifier<'a>>(&mut self, r#and: Z, op: impl Operator) {
        let (column_name, value) = r#and.value();

        let and_ = String::from(" OR ")
            + column_name
            + &op.as_str(self.params.len() + 1, &self.database_type);

        self.sql.push_str(&and_);
        self.params.push(value);
    }

    #[inline]
    fn order_by<Z: FieldIdentifier>(&mut self, order_by: Z, desc: bool) {
        self.sql.push_str(
            &(format!(
                " ORDER BY {}{}",
                order_by.as_str(),
                if desc { " DESC " } else { "" }
            )),
        );
    }
}
