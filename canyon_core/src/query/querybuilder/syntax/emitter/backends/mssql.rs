use crate::query::querybuilder::syntax::{
    ast::{delete::DeleteAst, insert::InsertAst, select::SelectAst, update::UpdateAst},
    dialect::MsSql,
    emitter::{
        EmitStep, SqlEmitter, types::delete::delete_default_plan, types::helpers, types::insert,
        types::select::select_default_plan, types::update::update_default_plan,
    },
};

#[derive(Default)]
pub struct SqlServerEmitter {}

impl<'a> SqlEmitter<'a, SelectAst<'a>> for SqlServerEmitter {
    type Dialect = MsSql;
    const PLAN: &'a [EmitStep<'a, SelectAst<'a>>] = select_default_plan!(Self::Dialect);
}

impl<'a> SqlEmitter<'a, InsertAst<'a>> for SqlServerEmitter {
    type Dialect = MsSql;

    const PLAN: &'a [EmitStep<'a, InsertAst<'a>>] = &[
        insert::__impl::emit_insert_into_keywords,
        |_ast, base_ast, tokens| helpers::emit_table::<Self::Dialect>(base_ast.table(), tokens),
        |ast, _base_ast, tokens| helpers::emit_unqualified_columns::<Self::Dialect>(&ast.columns, tokens),
        |ast, base_ast, tokens| __impl::emit_output::<Self::Dialect>(ast, base_ast, tokens),
        |ast, base_ast, tokens| insert::__impl::emit_values(ast, base_ast, tokens),
    ];
}
impl<'a> SqlEmitter<'a, UpdateAst<'a>> for SqlServerEmitter {
    type Dialect = MsSql;

    const PLAN: &'a [EmitStep<'a, UpdateAst<'a>>] = update_default_plan!(Self::Dialect);
}

impl<'a> SqlEmitter<'a, DeleteAst> for SqlServerEmitter {
    type Dialect = MsSql;

    const PLAN: &'a [EmitStep<'a, DeleteAst>] = delete_default_plan!(Self::Dialect);
}

mod __impl {
    use crate::query::querybuilder::syntax::emitter::types::helpers;
    use crate::query::querybuilder::syntax::{
        ast::{BaseAst, insert::InsertAst},
        dialect::SqlDialect,
        keyword::Keyword,
        tokens::SqlTokens,
    };

    pub(super) fn emit_output<'a, D>(
        ast: &InsertAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) where
        D: SqlDialect,
    {
        if ast.returning_columns.is_empty() {
            return;
        }

        tokens.keyword(Keyword::Output);

        for (index, column) in ast.returning_columns.iter().enumerate() {
            if index != 0 {
                tokens.comma();
            }

            tokens.keyword(Keyword::Inserted);
            tokens.dot();
            helpers::push_quoted_ident::<D, _>(column.name(), tokens);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::__impl::emit_output;
    use crate::query::querybuilder::syntax::{
        ast::{BaseAst, insert::InsertAst},
        column::ColumnRef,
        dialect::MsSql,
        tokens::SqlTokens,
        writer::TokenWriter,
    };

    fn render_output<'a>(ast: &'a InsertAst<'a>) -> String {
        let mut base_ast = BaseAst::default();
        let mut tokens = SqlTokens::default();

        emit_output::<MsSql>(ast, &mut base_ast, &mut tokens);

        TokenWriter::new()
            .render::<MsSql>(tokens)
            .expect("OUTPUT tokens should render successfully")
    }

    #[test]
    fn does_not_emit_output_when_returning_columns_are_empty() {
        let ast = InsertAst {
            returning_columns: vec![],
            ..Default::default()
        };

        assert_eq!(render_output(&ast), ";");
    }

    #[test]
    fn emits_output_for_one_returning_column() {
        let ast = InsertAst {
            returning_columns: vec![ColumnRef::from("id")],
            ..Default::default()
        };

        assert_eq!(render_output(&ast), "OUTPUT INSERTED.[id];");
    }

    #[test]
    fn emits_output_for_multiple_returning_columns() {
        let ast = InsertAst {
            returning_columns: vec![
                ColumnRef::from("id"),
                ColumnRef::from("created_at"),
                ColumnRef::from("updated_at"),
            ],
            ..Default::default()
        };

        assert_eq!(
            render_output(&ast),
            "OUTPUT INSERTED.[id], INSERTED.[created_at], INSERTED.[updated_at];"
        );
    }

    #[test]
    fn ignores_the_source_table_qualifier_in_returning_columns() {
        let ast = InsertAst {
            returning_columns: vec![
                ColumnRef::from("league.id"),
                ColumnRef::from("league.created_at"),
            ],
            ..Default::default()
        };

        assert_eq!(
            render_output(&ast),
            "OUTPUT INSERTED.[id], INSERTED.[created_at];"
        );
    }
}
