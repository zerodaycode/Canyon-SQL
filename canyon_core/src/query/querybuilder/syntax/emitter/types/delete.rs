macro_rules! delete_default_plan {
    ($dialect:ty) => {
        &[
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::delete::__impl::emit_delete_keyword(
                    ast,
                    base_ast,
                    tokens,
                )
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::delete::__impl::emit_from_keyword(
                    ast,
                    base_ast,
                    tokens,
                )
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::delete::__impl::emit_table::<$dialect>(
                    ast,
                    base_ast,
                    tokens,
                )
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::delete::__impl::emit_conditions::<$dialect>(
                    ast,
                    base_ast,
                    tokens,
                )
            },
        ]
    };
}

pub(crate) use delete_default_plan;

pub(crate) mod __impl {
    use crate::query::querybuilder::syntax::{
        ast::{BaseAst, delete::DeleteAst},
        dialect::SqlDialect,
        emitter::types::helpers,
        keyword::Keyword,
        tokens::SqlTokens,
    };

    pub(crate) fn emit_delete_keyword<'a>(
        _ast: &DeleteAst,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        tokens.keyword(Keyword::Delete);
    }

    pub(crate) fn emit_from_keyword<'a>(
        _ast: &DeleteAst,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        tokens.keyword(Keyword::From);
    }

    pub(crate) fn emit_table<'a, D>(
        _ast: &DeleteAst,
        base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) where
        D: SqlDialect,
    {
        helpers::emit_table::<D>(base_ast.table(), tokens);
    }

    pub(crate) fn emit_conditions<'a, D>(
        _ast: &DeleteAst,
        base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) where
        D: SqlDialect,
    {
        helpers::emit_query_conditions::<D>(base_ast.conditions(), tokens);
    }
}

#[cfg(test)]
mod tests {
    use crate::query::querybuilder::syntax::emitter::EmitStep;
    use crate::query::querybuilder::syntax::{
        ast::{BaseAst, delete::DeleteAst},
        dialect::{MsSql, PgDialect},
        emitter::{SqlEmitter, types::helpers::Range},
        writer::TokenWriter,
    };

    #[derive(Default)]
    struct TestDeleteEmitter;

    impl<'a> SqlEmitter<'a, DeleteAst> for TestDeleteEmitter {
        type Dialect = PgDialect;

        const PLAN: &'a [EmitStep<'a, DeleteAst>] = delete_default_plan!(Self::Dialect);
    }

    #[derive(Default)]
    struct TestDeleteEmitterMsSql;
    impl<'a> SqlEmitter<'a, DeleteAst> for TestDeleteEmitterMsSql {
        type Dialect = MsSql;

        const PLAN: &'a [EmitStep<'a, DeleteAst>] = delete_default_plan!(Self::Dialect);
    }

    fn render_standard<'a>(ast: &DeleteAst, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestDeleteEmitter;
        let tokens = emitter.emit(ast, base_ast);
        TokenWriter::new().render::<PgDialect>(tokens).unwrap()
    }

    fn render_mssql<'a>(ast: &DeleteAst, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestDeleteEmitterMsSql;
        let tokens = emitter.emit(ast, base_ast);
        TokenWriter::new().render::<MsSql>(tokens).unwrap()
    }

    #[test]
    fn emits_delete_from_table_without_conditions() {
        let ast = DeleteAst::default();

        let mut base_ast = BaseAst::new_ast("users".into());

        let sql = render_standard(&ast, &mut base_ast);
        assert_eq!(sql.trim(), "DELETE FROM \"users\";");
    }

    #[test]
    fn emits_delete_from_table_without_conditions_in_mssql() {
        let ast = DeleteAst::default();

        let mut base_ast = BaseAst::new_ast("users".into());

        let sql = render_mssql(&ast, &mut base_ast);
        assert_eq!(sql.trim(), "DELETE FROM [users];");
    }

    #[test]
    fn emits_delete_with_where_condition() {
        use crate::query::operators::Operator;
        use crate::query::querybuilder::syntax::clause::{ConditionClause, ConditionClauseKind};

        let ast = DeleteAst::default();

        let mut base_ast = BaseAst::new_ast("users".into());

        base_ast.add_condition(ConditionClause {
            kind: ConditionClauseKind::Where,
            column_name: "id".into(),
            operator: Operator::Eq,
            value_indexes: Some(Range::new_unbounded(3)),
        });

        let sql = render_standard(&ast, &mut base_ast);
        assert_eq!(sql.trim(), "DELETE FROM \"users\" WHERE \"id\" = $1;");
    }
}
