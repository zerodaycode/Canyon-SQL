#[cfg(any(feature = "postgres", feature = "mysql"))]
macro_rules! insert_default_plan {
    ($dialect:ty) => {
        &[
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::insert::__impl::emit_insert_into_keywords(
                    ast,
                    base_ast,
                    tokens,
                )
            },
            |_ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::helpers::emit_table::<$dialect>(
                    base_ast.table(),
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
                $crate::query::querybuilder::syntax::emitter::types::insert::__impl::emit_values(
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
    use crate::query::querybuilder::syntax::symbol::Symbol;
    use crate::query::querybuilder::syntax::{
        ast::insert::InsertAst, emitter::types::helpers, keyword::Keyword, tokens::SqlTokens,
    };

    #[cfg(any(feature = "postgres", feature = "mysql"))]
    use crate::query::querybuilder::syntax::dialect::SqlDialect;

    pub(crate) fn emit_insert_into_keywords<'a>(
        _ast: &InsertAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        tokens.keyword(Keyword::Insert);
        tokens.keyword(Keyword::Into);
    }

    #[cfg(any(feature = "postgres", feature = "mysql"))]
    pub(crate) fn emit_columns<'a, D: SqlDialect>(
        ast: &InsertAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        tokens.symbol(Symbol::LParen);
        helpers::emit_unqualified_columns::<D>(&ast.columns, tokens);
        tokens.symbol(Symbol::RParen);
    }

    pub(crate) fn emit_values<'a>(
        ast: &InsertAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        tokens.keyword(Keyword::Values);
        tokens.symbol(Symbol::LParen);
        helpers::emit_placeholders(&ast.columns, tokens);
        tokens.symbol(Symbol::RParen);
    }

    #[cfg(any(feature = "postgres", feature = "mysql"))]
    pub(crate) fn emit_returning<'a, D: SqlDialect>(
        ast: &InsertAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if !D::SUPPORTS_RETURNING || ast.returning_columns.is_empty() {
            return;
        }
        tokens.keyword(Keyword::Returning);
        helpers::emit_unqualified_columns::<D>(&ast.returning_columns, tokens)
    }
}

#[cfg(any(feature = "postgres", feature = "mysql"))]
pub(crate) use insert_default_plan;

#[cfg(test)]
mod tests {
    use crate::query::querybuilder::syntax::{
        ast::BaseAst,
        ast::insert::InsertAst,
        column::ColumnRef,
        dialect::{MySql, PgDialect},
        emitter::EmitStep,
        emitter::SqlEmitter,
        writer::TokenWriter,
    };

    #[derive(Default)]
    struct TestInsertEmitter;
    impl<'a> SqlEmitter<'a, InsertAst<'a>> for TestInsertEmitter {
        type Dialect = PgDialect;
        const PLAN: &'a [EmitStep<'a, InsertAst<'a>>] = insert_default_plan!(Self::Dialect);
    }

    #[derive(Default)]
    struct TestInsertEmitterNoReturning;
    impl<'a> SqlEmitter<'a, InsertAst<'a>> for TestInsertEmitterNoReturning {
        type Dialect = MySql;
        const PLAN: &'a [EmitStep<'a, InsertAst<'a>>] = insert_default_plan!(Self::Dialect);
    }

    fn col(name: &'_ str) -> ColumnRef<'_> {
        ColumnRef::from(name)
    }

    fn render_with_returning<'a>(ast: &InsertAst<'a>, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestInsertEmitter;
        let tokens = emitter.emit(ast, base_ast);
        TokenWriter::new().render::<PgDialect>(tokens).unwrap()
    }

    fn render_without_returning<'a>(ast: &InsertAst<'a>, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestInsertEmitterNoReturning;
        let tokens = emitter.emit(ast, base_ast);
        TokenWriter::new().render::<MySql>(tokens).unwrap()
    }

    #[test]
    fn emits_insert_columns_values_and_returning_when_supported() {
        let ast = InsertAst {
            columns: vec![col("id"), col("name")],
            returning_columns: vec![col("id")],
        };

        let mut base_ast = BaseAst::new_ast("users".into());

        let sql = render_with_returning(&ast, &mut base_ast);
        assert_eq!(
            sql,
            "INSERT INTO \"users\" (\"id\", \"name\") VALUES ($1, $2) RETURNING \"id\";"
        );
    }

    #[test]
    fn omits_returning_when_dialect_does_not_support_it() {
        let ast = InsertAst {
            columns: vec![col("id"), col("name")],
            returning_columns: vec![col("id")],
        };

        let mut base_ast = BaseAst::new_ast("users".into());

        let sql = render_without_returning(&ast, &mut base_ast);
        assert_eq!(
            sql.trim(),
            "INSERT INTO `users` (`id`, `name`) VALUES (?, ?);"
        );
    }

    #[test]
    fn emits_multiple_returning_columns_when_supported() {
        let ast = InsertAst {
            columns: vec![col("name"), col("email")],
            returning_columns: vec![col("id"), col("created_at")],
        };

        let mut base_ast = BaseAst::new_ast("users".into());

        let sql = render_with_returning(&ast, &mut base_ast);
        assert_eq!(
            sql.trim(),
            "INSERT INTO \"users\" (\"name\", \"email\") VALUES ($1, $2) RETURNING \"id\", \"created_at\";"
        );
    }

    #[test]
    fn does_not_emit_returning_keyword_when_returning_columns_are_empty_and_dialect_supports_returning()
     {
        let ast = InsertAst {
            columns: vec![col("name")],
            returning_columns: vec![],
        };

        let mut base_ast = BaseAst::new_ast("users".into());

        let sql = render_with_returning(&ast, &mut base_ast);
        assert_eq!(sql.trim(), "INSERT INTO \"users\" (\"name\") VALUES ($1);");
    }
}
