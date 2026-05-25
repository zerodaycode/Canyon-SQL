use crate::query::bounds::FieldIdentifier;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::tokens::{SqlToken, SqlTokens, ToSqlTokens};

#[derive(Debug, Clone, Default)]
pub struct ColumnRef<'a> {
    pub table: Option<&'a str>,
    pub column: &'a str,
    pub alias: Option<&'a str>,
}

impl<'a, T> From<T> for ColumnRef<'a>
where
    T: FieldIdentifier + 'a,
{
    fn from(value: T) -> Self {
        value.as_column_ref()
    }
}

impl<'a> From<&'a str> for ColumnRef<'a> {
    fn from(value: &'a str) -> Self {
        __impl::column_ref_from_str_ref(value)
    }
}

impl<'a, D: SqlDialect> ToSqlTokens<'a, D> for ColumnRef<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(__detail::calculate_column_ref_capacity(self));
        __impl::generate_column_ref_tokens::<D>(self, &mut out);
        out
    }
}

impl<'a> ColumnRef<'a> {
    pub const fn new(column_name: &'a str) -> Self {
        Self {
            column: column_name,
            table: None,
            alias: None,
        }
    }

    /// mutator that allows to modify a [`ColumnRef`] to have a <table>.<column> format
    ///
    /// Ex: SELECT * FROM <table>.<column> as <alias>
    pub fn table(mut self, table: &'a str) -> Self {
        self.table = Some(table);
        self
    }

    /// mutator that allows to modify a [`ColumnRef`] to have a AS clause for
    /// specify a SQL alias
    ///
    /// Ex: SELECT * FROM <table>.<column> as <alias>
    pub fn alias(mut self, alias: &'a str) -> Self {
        self.alias = Some(alias);
        self
    }
}

mod __impl {
    use crate::query::querybuilder::syntax::column::{__detail, ColumnRef};
    use crate::query::querybuilder::syntax::dialect::SqlDialect;
    use crate::query::querybuilder::syntax::emitter::types::helpers;
    use crate::query::querybuilder::syntax::keyword::Keyword;
    use crate::query::querybuilder::syntax::symbol::Symbol::Dot;
    use crate::query::querybuilder::syntax::tokens::SqlTokens;

    pub(crate) fn column_ref_from_str_ref(value: &str) -> ColumnRef<'_> {
        let trimmed = value.trim();

        let (before_alias, alias) = match __detail::find_case_insensitive_as(trimmed) {
            Some(idx) => {
                let (left, right) = trimmed.split_at(idx);
                let right = right[2..].trim_start();
                (left.trim(), Some(right.trim()))
            }
            None => (trimmed, None),
        };

        let (table, column) = match before_alias.split_once('.') {
            Some((tbl, col)) => (Some(tbl.trim()), col.trim()),
            None => (None, before_alias.trim()),
        };

        ColumnRef {
            table,
            column,
            alias,
        }
    }

    pub(crate) fn generate_column_ref_tokens<'a, D: SqlDialect>(
        __self: &ColumnRef<'a>,
        out: &mut SqlTokens<'a>,
    ) {
        if let Some(table_ref) = __self.table {
            helpers::push_quoted_ident::<D, _>(table_ref, out);
            out.symbol(Dot)
        }

        helpers::push_quoted_ident::<D, _>(__self.column, out);

        if let Some(alias) = __self.alias {
            out.keyword(Keyword::As);
            helpers::push_quoted_ident::<D, _>(alias, out);
        }
    }
}

mod __detail {
    use crate::query::querybuilder::syntax::column::ColumnRef;

    pub(crate) fn find_case_insensitive_as(s: &str) -> Option<usize> {
        let bytes = s.as_bytes();
        for i in 0..bytes.len().saturating_sub(2) {
            let a = bytes[i];
            let b = bytes[i + 1];

            // Match case-insensitive ASCII
            let is_a = a == b'a' || a == b'A';
            let is_s = b == b's' || b == b'S';

            if is_a && is_s {
                let before_ok = i > 0 && bytes[i - 1].is_ascii_whitespace();
                let after_ok = i + 2 < bytes.len() && bytes[i + 2].is_ascii_whitespace();

                if before_ok && after_ok {
                    return Some(i);
                }
            }
        }
        None
    }

    pub(crate) fn calculate_column_ref_capacity(__self: &ColumnRef) -> usize {
        let mut counter = 1; // at least the column name
        if __self.table.is_some() {
            counter += 2; // table name + dot
        }
        if __self.alias.is_some() {
            counter += 2; // AS + alias name
        }
        counter
    }
}

#[cfg(test)]
mod column_ref_from_str_tests {
    use super::ColumnRef;

    #[test]
    fn test_column_ref_simple_column() {
        let c = ColumnRef::from("name");
        assert_eq!(c.table, None);
        assert_eq!(c.column, "name");
        assert_eq!(c.alias, None);
    }

    #[test]
    fn test_column_ref_table_column() {
        let c = ColumnRef::from("users.name");
        assert_eq!(c.table, Some("users"));
        assert_eq!(c.column, "name");
        assert_eq!(c.alias, None);
    }

    #[test]
    fn test_column_ref_with_alias_uppercase_as() {
        let c = ColumnRef::from("users.name AS n");
        assert_eq!(c.table, Some("users"));
        assert_eq!(c.column, "name");
        assert_eq!(c.alias, Some("n"));
    }

    #[test]
    fn test_column_ref_with_alias_lowercase_as() {
        let c = ColumnRef::from("users.name as n");
        assert_eq!(c.table, Some("users"));
        assert_eq!(c.column, "name");
        assert_eq!(c.alias, Some("n"));
    }

    #[test]
    fn test_column_ref_with_alias_mixed_case_as() {
        let c = ColumnRef::from("users.name As n");
        assert_eq!(c.table, Some("users"));
        assert_eq!(c.column, "name");
        assert_eq!(c.alias, Some("n"));
    }

    #[test]
    fn test_column_ref_multiple_spaces_around_as() {
        let c = ColumnRef::from("users.name   AS    n");
        assert_eq!(c.table, Some("users"));
        assert_eq!(c.column, "name");
        assert_eq!(c.alias, Some("n"));
    }

    #[test]
    fn test_column_ref_alias_without_table() {
        let c = ColumnRef::from("name AS n");
        assert_eq!(c.table, None);
        assert_eq!(c.column, "name");
        assert_eq!(c.alias, Some("n"));
    }

    #[test]
    fn test_column_ref_no_alias_when_as_not_valid() {
        let c = ColumnRef::from("nameASn");
        assert_eq!(c.table, None);
        assert_eq!(c.column, "nameASn");
        assert_eq!(c.alias, None);
    }

    #[test]
    fn test_column_ref_trim_whitespace() {
        let c = ColumnRef::from("   users.name AS n   ");
        assert_eq!(c.table, Some("users"));
        assert_eq!(c.column, "name");
        assert_eq!(c.alias, Some("n"));
    }

    #[test]
    fn test_column_ref_alias_complex() {
        let c = ColumnRef::from("users.full_name AS fullNameAlias");
        assert_eq!(c.table, Some("users"));
        assert_eq!(c.column, "full_name");
        assert_eq!(c.alias, Some("fullNameAlias"));
    }

    #[test]
    fn test_column_ref_no_table_but_alias() {
        let c = ColumnRef::from("email AS e");
        assert_eq!(c.table, None);
        assert_eq!(c.column, "email");
        assert_eq!(c.alias, Some("e"));
    }

    #[test]
    fn test_column_ref_only_column_and_spaces() {
        let c = ColumnRef::from("   column_name   ");
        assert_eq!(c.table, None);
        assert_eq!(c.column, "column_name");
        assert_eq!(c.alias, None);
    }

    #[test]
    fn test_column_ref_only_table_column_with_spaces() {
        let c = ColumnRef::from("   users . name   ");
        assert_eq!(c.table, Some("users"));
        assert_eq!(c.column, "name");
        assert_eq!(c.alias, None);
    }
}

#[cfg(test)]
mod column_ref_alias_as_detection_tests {
    use crate::query::querybuilder::syntax::column::__detail::find_case_insensitive_as;

    #[test]
    fn test_find_as_basic_uppercase() {
        let idx = find_case_insensitive_as("col AS x").unwrap();
        assert_eq!(&"col AS x"[idx..idx + 2], "AS");
    }

    #[test]
    fn test_find_as_lowercase() {
        let idx = find_case_insensitive_as("col as x").unwrap();
        assert_eq!(&"col as x"[idx..idx + 2], "as");
    }

    #[test]
    fn test_find_as_mixed_case() {
        let idx = find_case_insensitive_as("col As x").unwrap();
        assert_eq!(&"col As x"[idx..idx + 2], "As");
    }

    #[test]
    fn test_find_as_with_multiple_spaces() {
        let idx = find_case_insensitive_as("col   AS    x").unwrap();
        assert_eq!(&"col   AS    x"[idx..idx + 2], "AS");
    }

    #[test]
    fn test_find_as_requires_space_before_and_after() {
        assert!(find_case_insensitive_as("colASx").is_none());
        assert!(find_case_insensitive_as("col ASx").is_none());
        assert!(find_case_insensitive_as("colAS x").is_none());
        assert!(find_case_insensitive_as("ASx").is_none());
        assert!(find_case_insensitive_as("xAS").is_none());
    }

    #[test]
    fn test_find_as_at_start_or_end() {
        assert!(find_case_insensitive_as(" AS x").is_some());
        assert!(find_case_insensitive_as("x AS ").is_some());
    }

    #[test]
    fn test_find_as_no_match() {
        assert!(find_case_insensitive_as("column something").is_none());
        assert!(find_case_insensitive_as("").is_none());
        assert!(find_case_insensitive_as("a s").is_none());
        assert!(find_case_insensitive_as("col AX x").is_none());
    }

    #[test]
    fn test_find_as_with_table_column() {
        let idx = find_case_insensitive_as("table.col as alias").unwrap();
        assert_eq!(&"table.col as alias"[idx..idx + 2], "as");
    }
}
