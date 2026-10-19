//! Native types (as a catalog reports them), logical types and how they compare.

use canyon_core::types::SqlType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Length {
    Chars(u32),
    Max,
}

/// SQL text the database itself reported (a column type, a default expression).
///
/// Only introspection builds it, and it never comes back from a snapshot: deserializing
/// one always fails, so no JSON can put SQL text into a statement.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct CatalogSql(String);

impl CatalogSql {
    // Built by introspection (reading catalogs comes next) and by tests
    #[allow(dead_code)]
    pub(crate) fn new(sql: impl Into<String>) -> Self {
        Self(sql.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for CatalogSql {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom(
            "catalog SQL is read from the database, never from a snapshot",
        ))
    }
}

/// A column type as the database reports it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NativeType {
    /// Lowercase catalog name: PG `typname`, SQL Server `sys.types.name`, MySQL `DATA_TYPE`.
    pub name: String,
    pub length: Option<Length>,
    /// Fractional-second digits for time types.
    pub precision: Option<u8>,
    /// The type as DDL text (`character varying(100)`, `nvarchar(max)`, `tinyint(1)`).
    pub raw: CatalogSql,
}

// Built by introspection (reading catalogs comes next) and by tests
#[allow(dead_code)]
impl NativeType {
    pub(crate) fn new(
        name: &str,
        length: Option<Length>,
        precision: Option<u8>,
        raw: &str,
    ) -> Self {
        Self {
            name: name.to_owned(),
            length,
            precision,
            raw: CatalogSql::new(raw),
        }
    }

    pub(crate) fn named(name: &str, raw: &str) -> Self {
        Self::new(name, None, None, raw)
    }
}

/// A column type mapped back to Canyon's types, or kept as-is when Canyon has no mapping.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Logical {
    Known(SqlType),
    Other(NativeType),
}

/// How an existing column relates to what a field needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compat {
    Exact,
    /// Holds every value and the driver reads and writes the field type from it.
    Wider,
    /// The driver handles it, but it can't hold every value. Warning only.
    Narrower,
    Incompatible,
}

/// Integers only match their exact width: the PostgreSQL and SQL Server drivers refuse
/// to read or write another width.
pub fn compat(column: &Logical, field: SqlType) -> Compat {
    let Logical::Known(column) = column else {
        return Compat::Incompatible;
    };
    match (*column, field) {
        (SqlType::Text { max_len: col }, SqlType::Text { max_len: wanted }) => {
            match (col, wanted) {
                (None, None) => Compat::Exact,
                (None, Some(_)) => Compat::Wider,
                (Some(_), None) => Compat::Narrower,
                (Some(c), Some(w)) if c == w => Compat::Exact,
                (Some(c), Some(w)) if c > w => Compat::Wider,
                _ => Compat::Incompatible,
            }
        }
        (SqlType::Time { precision: c }, SqlType::Time { precision: w })
        | (SqlType::Timestamp { precision: c }, SqlType::Timestamp { precision: w }) => {
            precision_compat(c, w)
        }
        (column, field) if column == field => Compat::Exact,
        _ => Compat::Incompatible,
    }
}

fn precision_compat(column: u8, wanted: u8) -> Compat {
    if column == wanted {
        Compat::Exact
    } else if column > wanted {
        Compat::Wider
    } else if wanted == SqlType::DEFAULT_PRECISION {
        // The field didn't ask for a precision: respect the column, warn
        Compat::Narrower
    } else {
        Compat::Incompatible
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Compat::*;
    use SqlType::*;

    fn k(t: SqlType) -> Logical {
        Logical::Known(t)
    }
    const fn text(n: Option<u32>) -> SqlType {
        Text { max_len: n }
    }

    #[test]
    fn integers_are_exact_only() {
        for (col, field, expected) in [
            (SmallInt, SmallInt, Exact),
            (Int, Int, Exact),
            (BigInt, BigInt, Exact),
            (Int, SmallInt, Incompatible),
            (BigInt, Int, Incompatible),
            (SmallInt, Int, Incompatible),
            (Int, BigInt, Incompatible),
        ] {
            assert_eq!(compat(&k(col), field), expected, "{col:?} for {field:?}");
        }
    }

    #[test]
    fn text_lengths() {
        for (col, field, expected) in [
            (text(None), text(None), Exact),
            (text(None), text(Some(100)), Wider),
            (text(Some(100)), text(None), Narrower),
            (text(Some(100)), text(Some(100)), Exact),
            (text(Some(200)), text(Some(100)), Wider),
            (text(Some(50)), text(Some(100)), Incompatible),
        ] {
            assert_eq!(compat(&k(col), field), expected, "{col:?} for {field:?}");
        }
    }

    #[test]
    fn temporal_precision() {
        let t = |p| Timestamp { precision: p };
        let tm = |p| Time { precision: p };
        for (col, field, expected) in [
            (t(6), t(6), Exact),
            (t(7), t(6), Wider),
            (t(0), t(6), Narrower), // default precision: warn only
            (t(3), t(3), Exact),
            (t(6), t(3), Wider),
            (t(0), t(3), Incompatible), // declared precision: the column must hold it
            (tm(7), tm(6), Wider),
            (tm(2), tm(6), Narrower),
        ] {
            assert_eq!(compat(&k(col), field), expected, "{col:?} for {field:?}");
        }
    }

    #[test]
    fn other_kinds_and_unknown_types_are_incompatible() {
        assert_eq!(compat(&k(Bool), Bool), Exact);
        assert_eq!(compat(&k(Date), Date), Exact);
        assert_eq!(compat(&k(Bool), SmallInt), Incompatible);
        assert_eq!(compat(&k(Date), Timestamp { precision: 6 }), Incompatible);
        assert_eq!(compat(&k(text(None)), Int), Incompatible);
        let xml = Logical::Other(NativeType::named("xml", "xml"));
        assert_eq!(compat(&xml, text(None)), Incompatible);
    }
}
