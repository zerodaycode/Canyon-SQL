use tokio_postgres::types::ToSql;

// TODO Docs
pub trait FieldIdentifier {
    fn value(self) -> String;
}

impl FieldIdentifier for &str {
    fn value(self) -> String {
        self.to_string()
    }
}

/// Bounds to some type T in order to make it callable over some fn parameter T
pub trait ForeignKeyable {
    /// Retrieves the field related to the column passed in
    fn get_fk_column(&self, column: &str) -> Option<String>;
}


/// To define trait objects that helps to relates the necessary bounds n the 'in_clause`
pub trait InClauseValues: ToSql + ToString {}