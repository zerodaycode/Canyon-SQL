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

        tokens.symbol(Symbol::LParen);
        helpers::emit_columns::<T::Dialect>(&ast.columns, &mut tokens);
        tokens.symbol(Symbol::RParen);

        tokens.keyword(Keyword::Values);
        tokens.symbol(Symbol::LParen);
        helpers::emit_placeholders::<T>(&ast.columns, base_ast, &mut tokens);
        tokens.symbol(Symbol::RParen);

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
    use crate::{
        query::{
            querybuilder::{
                syntax::{
                    ast::insert::InsertAst,
                    dialect::SqlDialect,
                    emitter::SqlEmitter,
                    emitter::types::helpers,
                    keyword::Keyword,
                    tokens::{PlaceholderKind, SqlToken, SqlTokens}
                }
            }
        }
    };

    pub(crate) fn emit_returning<'a, E: SqlEmitter<'a>>(
        ast: &InsertAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if !E::Dialect::SUPPORTS_RETURNING {
            return; // early guarding
        }
        tokens.keyword(Keyword::Returning);
        // add the columns
        helpers::emit_columns::<E::Dialect>(&ast.returning_columns, tokens)
    }
}
