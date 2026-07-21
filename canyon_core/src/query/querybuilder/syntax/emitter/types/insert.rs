macro_rules! insert_default_plan {
    ($dialect:ty) => {
        &[
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::insert::__impl::emit_insert_keyword(
                    ast,
                    base_ast,
                    tokens,
                )
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::insert::__impl::emit_into_keyword(
                    ast,
                    base_ast,
                    tokens,
                )
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::insert::__impl::emit_table::<$dialect>(
                    ast,
                    base_ast,
                    tokens,
                )
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::insert::__impl::emit_columns::<$dialect>(
                    ast,
                    base_ast,
                    tokens,
                )
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::insert::__impl::emit_values_keyword(
                    ast,
                    base_ast,
                    tokens,
                )
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::insert::__impl::emit_placeholders(
                    ast,
                    base_ast,
                    tokens,
                )
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::insert::__impl::emit_returning::<$dialect>(
                    ast,
                    base_ast,
                    tokens,
                )
            },
        ]
    }
}

pub(crate) mod __impl {
    use crate::query::querybuilder::syntax::ast::BaseAst;
    use crate::query::querybuilder::syntax::{
        ast::insert::InsertAst, dialect::SqlDialect, emitter::types::helpers, keyword::Keyword,
        tokens::SqlTokens,
    };

    pub(crate) fn emit_returning<'a, D: SqlDialect>(
        ast: &InsertAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if !D::SUPPORTS_RETURNING || ast.returning_columns.is_empty() {
            return;
        }
        tokens.keyword(Keyword::Returning);
        helpers::emit_columns::<D>(&ast.returning_columns, tokens)
    }
}

pub(crate) use insert_default_plan;

#[cfg(test)]
mod tests {
    use crate::query::querybuilder::syntax::dialect::{MsSql, PgDialect};
    use crate::query::querybuilder::syntax::emitter::{AstProcessor, EmitStep};
    use crate::query::querybuilder::syntax::query_kind::QueryKind;
    use crate::query::querybuilder::syntax::writer::TokenWriter;
    use crate::query::querybuilder::syntax::{
        ast::BaseAst, ast::insert::InsertAst, column::ColumnRef, dialect::StandardDialect,
        emitter::SqlEmitter,
    };

    #[derive(Default)]
    struct TestInsertEmitter;
    impl<'a> SqlEmitter<'a, InsertAst<'a>> for TestInsertEmitter {
        type Dialect = StandardDialect;
        const PLAN: &'a [EmitStep<'a, InsertAst<'a>>] = insert_default_plan!(Self::Dialect);
    }

    #[derive(Default)]
    struct TestInsertEmitterNoReturning;
    impl<'a> SqlEmitter<'a, InsertAst<'a>> for TestInsertEmitterNoReturning {
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
        let tokens = emitter.emit(ast, base_ast);
        TokenWriter::new().render::<PgDialect>(tokens).unwrap()
    }

    fn render_without_returning<'a>(
        ast: &SelectlessInsertAst<'a>,
        base_ast: &mut BaseAst<'a>,
    ) -> String {
        let mut emitter = TestInsertEmitterNoReturning;
        let tokens = emitter.emit(&ast.0, base_ast);
        TokenWriter::new().render::<MsSql>(tokens).unwrap()
    }

    /// Tiny wrapper only to keep helper signatures short in tests.
    #[derive(Default)]
    struct SelectlessInsertAst<'a>(InsertAst<'a>);

    impl<'a> SelectlessInsertAst<'a> {
        fn new(ast: InsertAst<'a>) -> Self {
            Self(ast)
        }
    }

    impl<'a> AstProcessor<'a> for SelectlessInsertAst<'a> {
        fn query_kind(&self) -> QueryKind {
            QueryKind::Insert
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
            "INSERT INTO \"users\" (\"id\", \"name\") VALUES ($1, $2) RETURNING \"id\";"
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
            "INSERT INTO [users] ([id], [name]) VALUES (@P1, @P2);"
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
            "INSERT INTO \"users\" (\"name\", \"email\") VALUES ($1, $2) RETURNING \"id\", \"created_at\";"
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
        assert_eq!(sql.trim(), "INSERT INTO \"users\" (\"name\") VALUES ($1);");
    }
}
