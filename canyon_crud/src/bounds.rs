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