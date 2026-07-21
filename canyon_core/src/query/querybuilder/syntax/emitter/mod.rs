pub(crate) mod backends;
pub(crate) mod types;

use crate::query::querybuilder::syntax::{
    ast::BaseAst, dialect::SqlDialect, query_kind::QueryKind, tokens::SqlTokens,
};
use transient::{Any, Inv};

// ---------- AST Processor marker trait ----------
pub trait AstProcessor<'a>: Default + AsAstProcessor<'a> {
    fn query_kind(&self) -> QueryKind;
}
/// Base trait for downcasting all the implementors of [`AstProcessor`] when they are hidden
/// behind an opaque type
pub trait AsAstProcessor<'a>: Any<Inv<'a>> {
    fn as_any(&self) -> &dyn Any<Inv<'a>>;
}

/// Blanket implementation for all the AST types
impl<'a, T> AsAstProcessor<'a> for T
where
    T: AstProcessor<'a>,
{
    fn as_any(&self) -> &dyn Any<Inv<'a>> {
        self
    }
}
// #[allow(type_alias_bounds)]
// type SelectStep<'a, E: SqlEmitter<'a, SelectAst<'a>> + 'a> = fn(&mut E, ast: &SelectAst<'a>, base_ast: &BaseAst<'a>);

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
