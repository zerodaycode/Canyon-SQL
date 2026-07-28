macro_rules! update_default_plan {
    ($dialect:ty) => {
        &[
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::update::__impl::emit_update_keyword(
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
                $crate::query::querybuilder::syntax::emitter::types::update::__impl::emit_set_keyword(
                    ast,
                    base_ast,
                    tokens,
                )
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::update::__impl::emit_set_clause::<$dialect>(
                    ast,
                    base_ast,
                    tokens,
                )
            },
            |_ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::helpers::emit_query_conditions::<$dialect>(
                    base_ast.conditions(),
                    tokens,
                )
            },
        ]
    };
}

pub(crate) use update_default_plan;

pub(crate) mod __impl {
    use crate::query::ColumnRef;
    use crate::query::querybuilder::syntax::symbol::Symbol;
    use crate::query::querybuilder::syntax::{
        ast::{BaseAst, update::UpdateAst},
        dialect::SqlDialect,
        keyword::Keyword,
        tokens::{SqlTokens, ToSqlTokens},
    };
    use crate::query::querybuilder::syntax::column::Qualification;
    use crate::query::querybuilder::syntax::emitter::types::helpers;

    pub(crate) fn emit_update_keyword<'a>(
        _ast: &UpdateAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        tokens.keyword(Keyword::Update);
    }

    pub(crate) fn emit_set_keyword<'a>(
        _ast: &UpdateAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        tokens.keyword(Keyword::Set);
    }

    pub(crate) fn emit_set_clause<'a, D: SqlDialect>(
        ast: &UpdateAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        for (i, col) in ast.columns.iter().enumerate() {
            if i > 0 {
                tokens.symbol(Symbol::Comma);
            }
            col.emit::<D>(Qualification::Unqualified, tokens);
            tokens.symbol(Symbol::Equals);
            tokens.placeholder();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::query::operators::Operator;
    use crate::query::querybuilder::syntax::clause::{ConditionClause, ConditionClauseKind};
    use crate::query::querybuilder::syntax::dialect::MsSql;
    use crate::query::querybuilder::syntax::emitter::EmitStep;
    use crate::query::querybuilder::syntax::emitter::types::helpers::Range;
    use crate::query::querybuilder::syntax::writer::TokenWriter;
    use crate::query::querybuilder::syntax::{
        ast::BaseAst, ast::update::UpdateAst, column::ColumnRef, dialect::StandardDialect,
        emitter::SqlEmitter,
    };

    #[derive(Default)]
    struct TestUpdateEmitter;
    impl<'a> SqlEmitter<'a, UpdateAst<'a>> for TestUpdateEmitter {
        type Dialect = StandardDialect;
        const PLAN: &'a [EmitStep<'a, UpdateAst<'a>>] = update_default_plan!(Self::Dialect);
    }

    #[derive(Default)]
    struct TestUpdateEmitterMsSql;
    impl<'a> SqlEmitter<'a, UpdateAst<'a>> for TestUpdateEmitterMsSql {
        type Dialect = MsSql;
        const PLAN: &'a [EmitStep<'a, UpdateAst<'a>>] = update_default_plan!(Self::Dialect);
    }

    fn col(name: &'_ str) -> ColumnRef<'_> {
        ColumnRef::from(name)
    }

    fn render_standard<'a>(ast: &UpdateAst<'a>, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestUpdateEmitter;
        let tokens = emitter.emit(ast, base_ast);
        TokenWriter::new()
            .render::<StandardDialect>(tokens)
            .unwrap()
    }

    fn render_mssql<'a>(ast: &UpdateAst<'a>, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestUpdateEmitterMsSql;
        let tokens = emitter.emit(ast, base_ast);
        TokenWriter::new().render::<MsSql>(tokens).unwrap()
    }

    #[test]
    fn emits_update_with_single_set_column() {
        let ast = UpdateAst {
            columns: vec![col("name")],
        };

        let mut base_ast = BaseAst::new_ast("users".into());

        let sql = render_standard(&ast, &mut base_ast);
        assert_eq!(sql, "UPDATE \"users\" SET \"name\" = $1;");
    }

    #[test]
    fn emits_update_with_multiple_set_columns() {
        let ast = UpdateAst {
            columns: vec![col("name"), col("email"), col("updated_at")],
        };

        let mut base_ast = BaseAst::new_ast("users".into());

        let sql = render_standard(&ast, &mut base_ast);

        assert_eq!(
            sql,
            "UPDATE \"users\" SET \"name\" = $1, \"email\" = $2, \"updated_at\" = $3;"
        );
    }

    #[test]
    fn emits_update_with_where_conditions() {
        let ast = UpdateAst {
            columns: vec![col("name"), col("email")],
        };

        let mut base_ast = BaseAst::new_ast("users".into());

        base_ast.add_condition(ConditionClause {
            kind: ConditionClauseKind::Where,
            column_name: "id".into(),
            operator: Operator::Eq,
            value_indexes: Some(Range::new_unbounded(3)),
        });

        let sql = render_standard(&ast, &mut base_ast);

        assert_eq!(
            sql,
            "UPDATE \"users\" SET \"name\" = $1, \"email\" = $2 WHERE \"id\" = $3;"
        );
    }

    #[test]
    fn emits_update_in_mssql_with_dialect_specific_identifiers_and_placeholders() {
        let ast = UpdateAst {
            columns: vec![col("name"), col("email")],
        };

        let mut base_ast = BaseAst::new_ast("users".into());

        let sql = render_mssql(&ast, &mut base_ast);

        assert_eq!(sql, "UPDATE [users] SET [name] = @P1, [email] = @P2;");
    }

    #[test]
    fn preserves_placeholder_sequence_between_set_and_where() {
        let ast = UpdateAst {
            columns: vec![col("name"), col("email")],
        };

        let mut base_ast = BaseAst::new_ast("users".into());

        base_ast.add_condition(ConditionClause {
            kind: ConditionClauseKind::Where,
            column_name: "id".into(),
            operator: Operator::Eq,
            value_indexes: Some(Range::new_unbounded(3)),
        });

        let sql = render_standard(&ast, &mut base_ast);

        assert_eq!(
            sql,
            "UPDATE \"users\" SET \"name\" = $1, \"email\" = $2 WHERE \"id\" = $3;"
        );
    }
}
