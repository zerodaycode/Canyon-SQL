use super::ModelError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// PostgreSQL's limit, the smallest of the supported backends.
pub const MAX_IDENT_LEN: usize = 63;

/// A table, column, schema or constraint name.
///
/// Deserializing validates like [`Ident::new`], so a name made by
/// [`Ident::from_catalog`] serializes but doesn't read back: only desired models go
/// into a snapshot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Ident(Box<str>);

impl Ident {
    /// A name Canyon will create: checked so it works the same on every backend.
    pub fn new(value: &str) -> Result<Self, ModelError> {
        let mut chars = value.chars();
        let valid_start = chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
        let valid_rest = chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
        if valid_start && valid_rest && value.len() <= MAX_IDENT_LEN {
            Ok(Self(value.into()))
        } else {
            Err(ModelError::InvalidIdent(value.to_owned()))
        }
    }

    /// A name read from a database catalog. Never checked: it is only ever quoted.
    pub fn from_catalog(value: &str) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Ident {
    type Error = ModelError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<Ident> for String {
    fn from(value: Ident) -> Self {
        value.0.into()
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// `schema.name`, or just `name` for the connection's default schema.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct QualifiedName {
    pub schema: Option<Ident>,
    pub name: Ident,
}

impl QualifiedName {
    pub fn new(schema: Option<&str>, name: &str) -> Result<Self, ModelError> {
        Ok(Self {
            schema: schema.map(Ident::new).transpose()?,
            name: Ident::new(name)?,
        })
    }

    /// `"table"` or `"schema.table"`, as written in `foreign_key(table = …)`.
    pub fn parse(dotted: &str) -> Result<Self, ModelError> {
        match dotted.split('.').collect::<Vec<_>>().as_slice() {
            [name] => Self::new(None, name),
            [schema, name] => Self::new(Some(schema), name),
            _ => Err(ModelError::InvalidIdent(dotted.to_owned())),
        }
    }
}

impl fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.schema {
            Some(schema) => write!(f, "{schema}.{}", self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_idents() {
        for ok in ["a", "_x", "player", "Player2", "a_b_c", &"x".repeat(63)] {
            assert!(Ident::new(ok).is_ok(), "{ok}");
        }
    }

    #[test]
    fn invalid_idents() {
        for bad in [
            "",
            "1abc",
            "a-b",
            "a b",
            "a\"b",
            "a]b",
            "a`b",
            "ñ",
            &"x".repeat(64),
        ] {
            assert!(
                matches!(Ident::new(bad), Err(ModelError::InvalidIdent(_))),
                "{bad}"
            );
        }
    }

    #[test]
    fn catalog_names_are_kept_as_they_are() {
        assert_eq!(
            Ident::from_catalog("PK__league__3213E83F").as_str(),
            "PK__league__3213E83F"
        );
        assert_eq!(Ident::from_catalog("My Table").as_str(), "My Table");
    }

    #[test]
    fn qualified_parse() {
        let t = QualifiedName::parse("league").unwrap();
        assert_eq!((t.schema, t.name.as_str()), (None, "league"));
        let t = QualifiedName::parse("s.league").unwrap();
        assert_eq!(t.schema.unwrap().as_str(), "s");
        assert!(QualifiedName::parse("a.b.c").is_err());
    }

    #[test]
    fn qualified_order_is_schema_then_name() {
        let mut v = [
            QualifiedName::parse("b").unwrap(),
            QualifiedName::parse("s.a").unwrap(),
            QualifiedName::parse("a").unwrap(),
        ];
        v.sort();
        let names: Vec<_> = v.iter().map(ToString::to_string).collect();
        assert_eq!(names, ["a", "b", "s.a"]);
    }

    #[test]
    fn deserializing_goes_through_validation() {
        // The snapshot JSON can't smuggle a bad name in
        assert!(serde_json::from_str::<Ident>("\"a-b\"").is_err());
        let ok = Ident::new("ok").unwrap();
        let json = serde_json::to_string(&ok).unwrap();
        assert_eq!(serde_json::from_str::<Ident>(&json).unwrap(), ok);
        // Catalog names don't come back: only desired models go into a snapshot
        let catalog = serde_json::to_string(&Ident::from_catalog("My Table")).unwrap();
        assert!(serde_json::from_str::<Ident>(&catalog).is_err());
    }
}
