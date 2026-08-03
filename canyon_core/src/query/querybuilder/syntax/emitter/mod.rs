pub(crate) mod backends;
pub(crate) mod types;

use crate::connection::database_type::DatabaseType;
use crate::query::querybuilder::syntax::emitter::backends::{
    MySqlEmitter, PgEmitter, SqlServerEmitter,
};
use crate::query::querybuilder::syntax::{
    ast::BaseAst, dialect::SqlDialect, query_kind::QueryKind, tokens::SqlTokens,
};

// ---------- AST Processor marker trait ----------
pub trait AstProcessor<'a>: Default {
    fn query_kind(&self) -> QueryKind;
}

pub type EmitStep<'a, P> = fn(&P, &mut BaseAst<'a>, &mut SqlTokens<'a>);

pub trait BackendEmittable<'a>: AstProcessor<'a> {
    fn emit_for(
        database_type: DatabaseType,
        ast: &Self,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a>;
}
impl<'a, P> BackendEmittable<'a> for P
where
    P: AstProcessor<'a> + 'a,
    PgEmitter: SqlEmitter<'a, P>,
    MySqlEmitter: SqlEmitter<'a, P>,
    SqlServerEmitter: SqlEmitter<'a, P>,
{
    fn emit_for(
        database_type: DatabaseType,
        ast: &Self,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a> {
        match database_type {
            DatabaseType::PostgreSql | DatabaseType::Deferred => {
                PgEmitter::default().emit(ast, base_ast)
            }
            DatabaseType::MySQL => MySqlEmitter::default().emit(ast, base_ast),
            DatabaseType::SqlServer => SqlServerEmitter::default().emit(ast, base_ast),
        }
    }
}
pub trait SqlEmitter<'a, P>
where
    Self: Sized,
    P: AstProcessor<'a> + 'a,
{
    type Dialect: SqlDialect;

    /// Emit SQL tokens for the given AST node and table metadata.
    ///
    /// This method is the central entry point for query emission.
    /// Given:
    /// - an AST of type `P` representing the query structure,
    /// - a `TableMetadata` reference describing the target table,
    /// the emitter must generate the appropriate SQL tokens into its
    /// internal buffer.
    ///
    /// # Semantics
    ///
    /// The implementation of `emit` must produce tokens **in the correct
    /// order** determined by the semantics of `P` and the rules of the
    /// selected SQL dialect (`Self::Dialect`). For example, a `SELECT`
    /// emission writes: `SELECT ... FROM ... WHERE ...`. A backend
    /// emitter may choose to include or omit certain clauses (e.g.,
    /// `RETURNING`) depending on dialect support.
    const PLAN: &'a [EmitStep<'a, P>];

    #[inline]
    fn emit(&mut self, ast: &P, base_ast: &mut BaseAst<'a>) -> SqlTokens<'a> {
        let mut tokens = SqlTokens::default();

        for step in Self::PLAN {
            step(ast, base_ast, &mut tokens)
        }

        tokens
    }
}
