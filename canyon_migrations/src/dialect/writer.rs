use super::DdlDialect;
use crate::model::{CatalogSql, Ident, QualifiedName};
use crate::op::{Param, Statement};
use canyon_core::query::querybuilder::syntax::dialect::PlaceholderSymbol;
use std::marker::PhantomData;

/// A column type as SQL text. Only dialects build it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSql(String);

impl TypeSql {
    pub(crate) fn new(sql: String) -> Self {
        Self(sql)
    }

    #[cfg(any(feature = "mssql", feature = "mysql"))]
    pub(crate) fn from_catalog(sql: &CatalogSql) -> Self {
        Self(sql.as_str().to_owned())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Builds one statement. Text goes in only as `&'static str`, names only quoted,
/// values only as parameters.
pub struct DdlWriter<D: DdlDialect> {
    sql: String,
    params: Vec<Param>,
    dialect: PhantomData<D>,
}

impl<D: DdlDialect> Default for DdlWriter<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D: DdlDialect> DdlWriter<D> {
    pub fn new() -> Self {
        Self {
            sql: String::new(),
            params: Vec::new(),
            dialect: PhantomData,
        }
    }

    pub fn raw(&mut self, text: &'static str) -> &mut Self {
        self.sql.push_str(text);
        self
    }

    pub fn ident(&mut self, ident: &Ident) -> &mut Self {
        self.sql.push_str(&D::IDENT_QUOTING.quote(ident.as_str()));
        self
    }

    pub fn qualified(&mut self, name: &QualifiedName) -> &mut Self {
        if let Some(schema) = &name.schema {
            self.ident(schema).raw(".");
        }
        self.ident(&name.name)
    }

    pub fn idents(&mut self, idents: &[Ident]) -> &mut Self {
        for (i, ident) in idents.iter().enumerate() {
            if i > 0 {
                self.raw(", ");
            }
            self.ident(ident);
        }
        self
    }

    pub fn ty(&mut self, ty: &TypeSql) -> &mut Self {
        self.sql.push_str(ty.as_str());
        self
    }

    /// SQL text the database itself reported (a type or a default expression).
    pub(crate) fn catalog_sql(&mut self, sql: &CatalogSql) -> &mut Self {
        self.sql.push_str(sql.as_str());
        self
    }

    /// `value` as an escaped string literal, for data that has to be in the DDL text
    /// (a default value). Everything else goes in as a parameter.
    pub(crate) fn literal(&mut self, value: &str) -> &mut Self {
        self.sql.push_str(&D::text_literal(value));
        self
    }

    pub fn param(&mut self, value: Param) -> &mut Self {
        self.params.push(value);
        let n = self.params.len();
        match D::PLACEHOLDER_SYMBOL {
            PlaceholderSymbol::QuestionMark => self.sql.push('?'),
            symbol => {
                use std::fmt::Write;
                let _ = write!(self.sql, "{symbol}{n}");
            }
        }
        self
    }

    /// The statement, transactional when the backend's DDL is.
    pub fn finish(self) -> Statement {
        let transactional = D::TRANSACTIONAL_DDL;
        self.finish_with(transactional)
    }

    pub fn finish_with(self, transactional: bool) -> Statement {
        Statement {
            sql: self.sql,
            params: self.params,
            transactional,
        }
    }

    /// Just the text, for a quoted name passed as a parameter value.
    pub(crate) fn into_sql(self) -> String {
        self.sql
    }
}

#[cfg(feature = "mysql")]
impl DdlWriter<canyon_core::query::querybuilder::syntax::dialect::MySql> {
    /// A MySQL `COMMENT` string: it takes no hex literal (error 1064), so quotes are
    /// doubled, and backslashes too unless `NO_BACKSLASH_ESCAPES` makes them plain.
    pub(crate) fn comment(&mut self, value: &str, no_backslash_escapes: bool) -> &mut Self {
        self.sql.push('\'');
        for c in value.chars() {
            match c {
                '\'' => self.sql.push_str("''"),
                '\\' if !no_backslash_escapes => self.sql.push_str("\\\\"),
                c => self.sql.push(c),
            }
        }
        self.sql.push('\'');
        self
    }
}

// Compares the three dialects, so it needs all of them
#[cfg(all(test, feature = "postgres", feature = "mssql", feature = "mysql"))]
mod tests {
    use super::*;
    use crate::model::{Ident, QualifiedName};
    use canyon_core::query::querybuilder::syntax::dialect::{MsSql, MySql, PgDialect};

    fn q(s: &str) -> QualifiedName {
        QualifiedName::parse(s).unwrap()
    }

    #[test]
    fn idents_are_quoted_per_dialect() {
        let mut pg = DdlWriter::<PgDialect>::new();
        pg.raw("DROP TABLE ").qualified(&q("s.player"));
        assert_eq!(pg.finish().sql, "DROP TABLE \"s\".\"player\"");

        let mut ms = DdlWriter::<MsSql>::new();
        ms.raw("DROP TABLE ").qualified(&q("player"));
        assert_eq!(ms.finish().sql, "DROP TABLE [player]");

        let mut my = DdlWriter::<MySql>::new();
        my.raw("DROP TABLE ").qualified(&q("player"));
        assert_eq!(my.finish().sql, "DROP TABLE `player`");
    }

    #[test]
    fn catalog_names_are_escaped() {
        let mut ms = DdlWriter::<MsSql>::new();
        ms.raw("DROP TABLE ").ident(&Ident::from_catalog("we]ird"));
        assert_eq!(ms.finish().sql, "DROP TABLE [we]]ird]");
    }

    #[test]
    fn params_are_numbered_per_dialect() {
        let mut pg = DdlWriter::<PgDialect>::new();
        pg.raw("SELECT ")
            .param(Param::Text("a".into()))
            .raw(", ")
            .param(Param::Int(1));
        let st = pg.finish();
        assert_eq!(st.sql, "SELECT $1, $2");
        assert_eq!(st.params, [Param::Text("a".into()), Param::Int(1)]);

        let mut ms = DdlWriter::<MsSql>::new();
        ms.raw("EXEC sp_rename ")
            .param(Param::Text("a".into()))
            .raw(", ")
            .param(Param::Text("b".into()));
        assert_eq!(ms.finish().sql, "EXEC sp_rename @P1, @P2");

        let mut my = DdlWriter::<MySql>::new();
        my.raw("SELECT ").param(Param::Int(1));
        assert_eq!(my.finish().sql, "SELECT ?");
    }

    #[test]
    fn ident_lists() {
        let mut pg = DdlWriter::<PgDialect>::new();
        pg.idents(&[Ident::new("a").unwrap(), Ident::new("b").unwrap()]);
        assert_eq!(pg.finish().sql, "\"a\", \"b\"");
    }
}
