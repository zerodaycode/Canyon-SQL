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

/// A strategy for emitting SQL text from a specific query AST type.
///
/// `SqlEmitter` represents a backend-specific SQL generator that knows how
/// to produce SQL tokens for a given AST node type `P` in a particular
/// dialect.
///
/// The emitter:
/// - accumulates SQL tokens (`SqlTokens`) during emission,
/// - knows its SQL dialect (`Dialect`) which governs syntax details,
/// - and produces SQL text deterministically given an AST of type `P`
///   and its associated `TableMetadata`.
///
/// This trait is **unified**: it combines both the emitter infrastructure
/// (token buffer access and dialect information) and the actual
/// emission behavior for the AST `P`. That eliminates the need for
/// separate “support traits” such as `EmitSelect`, `EmitInsert`, etc.
///
/// Users of this trait instantiate a backend emitter (e.g., `PgEmitter`)
/// and call `emit` once with the appropriate AST and table metadata. The
/// emitter pushes tokens into its internal buffer. After emission,
/// the tokens can be rendered into a final SQL string by a writer (e.g.,
/// `TokenWriter`).
///
/// # Type Parameters
///
/// - `P`: the type of the query AST being emitted (e.g., `SelectAst`, `InsertAst`, etc.).
///
/// # Example
///
/// ```rust
/// let mut tokens = SqlTokens::default();
/// let mut emitter: PgEmitter<'_, SelectAst<'_>> = PgEmitter::new(&mut tokens);
/// emitter.emit(&select_ast, &table_metadata);
/// let sql = TokenWriter::new().render(emitter.tokens(), database_type)?;
/// ```
///
/// The example shows emission for a `SELECT` query in PostgreSQL.
// pub type EmitStep<'a, P> =
//     for<'step> fn(&'step P, &'step mut BaseAst<'a>, &'step mut SqlTokens<'a>);

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
