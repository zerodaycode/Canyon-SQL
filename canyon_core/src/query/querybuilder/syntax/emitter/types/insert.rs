use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::ToSqlTokens;
use crate::query::querybuilder::syntax::{
    ast::BaseAst,
    ast::insert::InsertAst,
    emitter::types::helpers,
    emitter::{AstProcessor, SqlEmitter},
    keyword::Keyword,
    symbol::Symbol,
    tokens::SqlTokens,
};

impl<'a, T> EmitInsert<'a> for T
where
    T: SqlEmitter<'a>,
{
    fn emit_insert(
        &mut self,
        ast: &impl AstProcessor<'a>,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a> {
        let mut tokens = SqlTokens::default();

        let ast = transient::Downcast::downcast_ref::<InsertAst>(ast.as_any()).expect(
            "[emitInsert] - Handle this propagating result and introducing custom error types",
        );

        tokens.keyword(Keyword::Insert);
        tokens.keyword(Keyword::Into);

        tokens
            .extend(<TableMetadata<'_> as ToSqlTokens<'_, T::Dialect>>::to_tokens(&base_ast.table));
        tokens.whitespace();

        tokens.symbol(Symbol::LParen);
        helpers::emit_columns::<T::Dialect>(&ast.columns, &mut tokens);
        tokens.symbol(Symbol::RParen);
        tokens.whitespace();

        tokens.keyword(Keyword::Values);
        tokens.symbol(Symbol::LParen);
        helpers::emit_placeholders(&ast.columns, base_ast, &mut tokens);
        tokens.symbol(Symbol::RParen);
        tokens.whitespace();

        __impl::emit_returning::<T>(ast, &mut tokens);

        tokens
    }
}

pub trait EmitInsert<'a>: SqlEmitter<'a> {
    fn emit_insert(
        &mut self,
        ast: &impl AstProcessor<'a>,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a>;
}

pub(crate) mod __impl {
    use crate::query::querybuilder::syntax::{
        ast::insert::InsertAst, dialect::SqlDialect, emitter::SqlEmitter, emitter::types::helpers,
        keyword::Keyword, tokens::SqlTokens,
    };

    pub(crate) fn emit_returning<'a, E: SqlEmitter<'a>>(
        ast: &InsertAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if !E::Dialect::SUPPORTS_RETURNING || ast.returning_columns.is_empty() {
            return; // early guarding
        }
        tokens.keyword(Keyword::Returning);
        // add the columns
        helpers::emit_columns::<E::Dialect>(&ast.returning_columns, tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::EmitInsert;
    use crate::query::querybuilder::syntax::dialect::MsSql;
    use crate::query::querybuilder::syntax::writer::TokenWriter;
    use crate::query::querybuilder::syntax::{
        ast::BaseAst, ast::insert::InsertAst, column::ColumnRef, dialect::StandardDialect,
        emitter::SqlEmitter,
    };

    #[derive(Default)]
    struct TestInsertEmitter;
    impl<'a> SqlEmitter<'a> for TestInsertEmitter {
        type Dialect = StandardDialect;
    }

    #[derive(Default)]
    struct TestInsertEmitterNoReturning;
    impl<'a> SqlEmitter<'a> for TestInsertEmitterNoReturning {
        type Dialect = MsSql;
    }

    fn col(name: &'_ str) -> ColumnRef<'_> {
        ColumnRef::from(name)
    }

    fn render_with_returning<'a>(
        ast: &SelectlessInsertAst<'a>,
        base_ast: &mut BaseAst<'a>,
    ) -> String {
        let mut emitter = TestInsertEmitter;
        let mut tokens = emitter.emit_insert(&ast.0, base_ast);
        TokenWriter::new()
            .render::<TestInsertEmitter>(&mut tokens)
            .unwrap()
    }

    fn render_without_returning<'a>(
        ast: &SelectlessInsertAst<'a>,
        base_ast: &mut BaseAst<'a>,
    ) -> String {
        let mut emitter = TestInsertEmitterNoReturning;
        let mut tokens = emitter.emit_insert(&ast.0, base_ast);
        TokenWriter::new()
            .render::<TestInsertEmitterNoReturning>(&mut tokens)
            .unwrap()
    }

    /// Tiny wrapper only to keep helper signatures short in tests.
    struct SelectlessInsertAst<'a>(InsertAst<'a>);

    impl<'a> SelectlessInsertAst<'a> {
        fn new(ast: InsertAst<'a>) -> Self {
            Self(ast)
        }
    }

    #[test]
    fn emits_insert_columns_values_and_returning_when_supported() {
        let ast = SelectlessInsertAst::new(InsertAst {
            columns: vec![col("id"), col("name")],
            returning_columns: vec![col("id")],
        });

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render_with_returning(&ast, &mut base_ast);
        assert_eq!(
            sql,
            "INSERT INTO users (\"id\", \"name\") VALUES ($1, $2) RETURNING \"id\""
        );
    }

    #[test]
    fn omits_returning_when_dialect_does_not_support_it() {
        let ast = SelectlessInsertAst::new(InsertAst {
            columns: vec![col("id"), col("name")],
            returning_columns: vec![col("id")],
        });

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render_without_returning(&ast, &mut base_ast);
        assert_eq!(
            sql.trim(),
            "INSERT INTO users ([id], [name]) VALUES (@P1, @P2)"
        );
    }

    #[test]
    fn emits_multiple_returning_columns_when_supported() {
        let ast = SelectlessInsertAst::new(InsertAst {
            columns: vec![col("name"), col("email")],
            returning_columns: vec![col("id"), col("created_at")],
        });

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render_with_returning(&ast, &mut base_ast);
        assert_eq!(
            sql.trim(),
            "INSERT INTO users (\"name\", \"email\") VALUES ($1, $2) RETURNING \"id\", \"created_at\""
        );
    }

    #[test]
    fn does_not_emit_returning_keyword_when_returning_columns_are_empty_and_dialect_supports_returning()
     {
        let ast = SelectlessInsertAst::new(InsertAst {
            columns: vec![col("name")],
            returning_columns: vec![],
        });

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render_with_returning(&ast, &mut base_ast);
        assert_eq!(sql.trim(), "INSERT INTO users (\"name\") VALUES ($1)");
    }
}
