use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::ast::insert::InsertAst;
use crate::query::querybuilder::syntax::emitter::{AstProcessor, SqlEmitter};
use crate::query::querybuilder::syntax::emitter::types::helpers;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::symbol::Symbol;
use crate::query::querybuilder::syntax::tokens::SqlTokens;

impl<'a, T> EmitInsert<'a> for T
where
    T: SqlEmitter<'a>,
{
    fn emit_insert(
        &mut self,
        ast: &impl AstProcessor<'a>,
        _base_ast: &BaseAst<'a>,
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

        tokens
    }
}

pub trait EmitInsert<'a>: SqlEmitter<'a> {
    fn emit_insert(&mut self,
                   ast: &impl AstProcessor<'a>,
                   base_ast: &BaseAst<'a>) -> SqlTokens<'a>;
}

pub(crate) mod __impl {
    use crate::query::querybuilder::syntax::ast::insert::InsertAst;
    use crate::query::querybuilder::syntax::dialect::SqlDialect;
    use crate::query::querybuilder::syntax::emitter::SqlEmitter;
    use crate::query::querybuilder::syntax::emitter::types::helpers;
    use crate::query::querybuilder::syntax::keyword::Keyword;
    use crate::query::querybuilder::syntax::tokens::SqlTokens;

    pub(crate) fn emit_returning<'a, E: SqlEmitter<'a>>(
        ast: &InsertAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if !E::Dialect::SUPPORTS_RETURNING {
            return; // early guarding
        }
        tokens.keyword(Keyword::Returning);
        // add the columns
        helpers::emit_columns::<E::Dialect>(&ast.columns, tokens)
    }
}
