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

pub(crate) fn general_insert_impl<'a, T, P>(
    ast: &P,
    base_ast: &mut BaseAst<'a>,
) -> SqlTokens<'a>
where
    T: SqlEmitter<'a, P>,
    P: AstProcessor<'a>,
{
    let mut tokens = SqlTokens::default();

    let ast = transient::Downcast::downcast_ref::<InsertAst>(ast.as_any()).expect(
        "[emitInsert] - Handle this propagating result and introducing custom error types",
    );

    tokens.keyword(Keyword::Insert);
    tokens.keyword(Keyword::Into);

    tokens
        .extend(<TableMetadata<'_> as ToSqlTokens<'_, T::Dialect>>::to_tokens(&base_ast.table));

    tokens.symbol(Symbol::LParen);
    helpers::emit_columns::<T::Dialect>(&ast.columns, &mut tokens);
    tokens.symbol(Symbol::RParen);

    tokens.keyword(Keyword::Values);
    tokens.symbol(Symbol::LParen);
    helpers::emit_placeholders(&ast.columns, &mut tokens);
    tokens.symbol(Symbol::RParen);

    __impl::emit_returning::<T::Dialect>(ast, &mut tokens);

    tokens
}

// impl<'a, T, P> EmitInsert<'a, T, P> for T
// where
//     T: SqlEmitter<'a, P>,
//     P: AstProcessor<'a>,
// {
//     fn emit_insert(
//         &mut self,
//         ast: &P,
//         base_ast: &mut BaseAst<'a>,
//     ) -> SqlTokens<'a> {
//         let mut tokens = SqlTokens::default();
// 
//         let ast = transient::Downcast::downcast_ref::<InsertAst>(ast.as_any()).expect(
//             "[emitInsert] - Handle this propagating result and introducing custom error types",
//         );
// 
//         tokens.keyword(Keyword::Insert);
//         tokens.keyword(Keyword::Into);
// 
//         tokens
//             .extend(<TableMetadata<'_> as ToSqlTokens<'_, T::Dialect>>::to_tokens(&base_ast.table));
// 
//         tokens.symbol(Symbol::LParen);
//         helpers::emit_columns::<T::Dialect>(&ast.columns, &mut tokens);
//         tokens.symbol(Symbol::RParen);
// 
//         tokens.keyword(Keyword::Values);
//         tokens.symbol(Symbol::LParen);
//         helpers::emit_placeholders(&ast.columns, &mut tokens);
//         tokens.symbol(Symbol::RParen);
// 
//         __impl::emit_returning::<T::Dialect>(ast, &mut tokens);
// 
//         tokens
//     }
// }

pub trait EmitInsert<'a, T, P>
where
    T: SqlEmitter<'a, P>,
    P: AstProcessor<'a>,
{
    fn emit_insert(
        &mut self,
        ast: &P,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a>;
}

pub(crate) mod __impl {
    use crate::query::querybuilder::syntax::{
        ast::insert::InsertAst, dialect::SqlDialect, emitter::types::helpers,
        keyword::Keyword, tokens::SqlTokens,
    };
    use crate::query::querybuilder::syntax::emitter::AstProcessor;

    pub(crate) fn emit_returning<'a, D: SqlDialect>(
        ast: &InsertAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if !D::SUPPORTS_RETURNING || ast.returning_columns.is_empty() {
            return;
        }
        tokens.keyword(Keyword::Returning);
        helpers::emit_columns::<D>(&ast.returning_columns, tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::EmitInsert;
    use crate::query::querybuilder::syntax::dialect::{MsSql, PgDialect};
    use crate::query::querybuilder::syntax::writer::TokenWriter;
    use crate::query::querybuilder::syntax::{
        ast::BaseAst, ast::insert::InsertAst, column::ColumnRef, dialect::StandardDialect,
        emitter::SqlEmitter,
    };
    use crate::query::querybuilder::syntax::emitter::AstProcessor;
    use crate::query::querybuilder::syntax::query_kind::QueryKind;

    #[derive(Default)]
    struct TestInsertEmitter;
    impl<'a, P: AstProcessor<'a>> SqlEmitter<'a, P> for TestInsertEmitter {
        type Dialect = StandardDialect;
    }

    #[derive(Default)]
    struct TestInsertEmitterNoReturning;
    impl<'a> SqlEmitter<'a, P> for TestInsertEmitterNoReturning {
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
        TokenWriter::new()
            .render::<PgDialect>(tokens)
            .unwrap()
    }

    fn render_without_returning<'a>(
        ast: &SelectlessInsertAst<'a>,
        base_ast: &mut BaseAst<'a>,
    ) -> String {
        let mut emitter = TestInsertEmitterNoReturning;
        let tokens = emitter.emit(&ast.0, base_ast);
        TokenWriter::new()
            .render::<MsSql>(tokens)
            .unwrap()
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
