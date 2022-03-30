// TODO Docs
pub trait FieldIdentifier {
    fn value(self) -> String;
}

impl FieldIdentifier for &str {
    fn value(self) -> String {
        self.to_string()
    }
}