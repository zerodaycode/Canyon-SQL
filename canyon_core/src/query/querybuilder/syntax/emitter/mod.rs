pub(crate) mod backends;
mod types;

use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::ast::delete::DeleteAst;
use crate::query::querybuilder::syntax::ast::insert::InsertAst;
use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::ast::update::UpdateAst;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::emitter::types::select::EmitSelect;
use crate::query::querybuilder::syntax::query_kind::QueryKind;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::SqlTokens;


// ---------- AST Processor marker trait ----------
pub trait AstProcessor: Default {
    // TODO: get base? as mut ref for convenience?
    fn query_kind(&self) -> QueryKind;
} // TODO: maybe this and the other one are visitor related?



// #[allow(type_alias_bounds)]
// type SelectStep<'a, E: SqlEmitter<'a, SelectAst<'a>> + 'a> = fn(&mut E, ast: &SelectAst<'a>, base_ast: &BaseAst);


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
pub trait SqlEmitter<'a, P: AstProcessor> {
    /// The [`SqlDialect`] used by this emitter.
    ///
    /// Each backend emitter selects a dialect type that implements
    /// [`SqlDialect`]. This associated type must be distinct for each
    /// backend so that syntax differences can be encoded in the type
    /// system.
    type Dialect: SqlDialect;

    /// Mutable access to the internal SQL token buffer.
    ///
    /// SQL emission produces a sequence of tokens such as keywords,
    /// symbols, identifiers, and parameter placeholders. Emitters write
    /// these tokens into the buffer during `emit`. After emission is
    /// complete, the buffer can be passed to a renderer to produce
    /// a final SQL string.
    fn tokens(&mut self) -> &mut SqlTokens<'a>;

    /// Emit SQL tokens for the given AST node and table metadata.
    ///
    /// This method is the central entry point for query emission.
    /// Given:
    /// - an AST of type `P` representing the query structure,
    /// - a `TableMetadata` reference describing the target table,
    /// the emitter must generate the appropriate SQL tokens into its
    /// internal buffer.
    ///
    /// # Parameters
    /// TODO:
    /// - `ast`: A reference to the query AST to be emitted.
    /// - `table_metadata`: A reference containing table name and other
    ///   basic metadata required for correctly forming qualified database
    ///   identifiers such as table and schema names.
    ///
    /// # Semantics
    ///
    /// The implementation of `emit` must produce tokens **in the correct
    /// order** determined by the semantics of `P` and the rules of the
    /// selected SQL dialect (`Self::Dialect`). For example, a `SELECT`
    /// emission writes: `SELECT ... FROM ... WHERE ...`. A backend
    /// emitter may choose to include or omit certain clauses (e.g.,
    /// `RETURNING`) depending on dialect support.
    fn emit(
        &mut self,
        ast: &'a P,
        base_ast: &'a BaseAst<'a>
    );// TODO: should emit as the outer wrapper really return the emitter internal buffer?
}

pub trait EmitInsert<'a>: SqlEmitter<'a, InsertAst<'a>> {
    fn emit_insert(&mut self, ast: &'a InsertAst<'a>, meta: &TableMetadata<'a>);
}

pub trait EmitUpdate<'a>: SqlEmitter<'a, UpdateAst<'a>> {
    fn emit_update(&mut self, ast: &'a UpdateAst<'a>, meta: &TableMetadata<'a>);
}

pub trait EmitDelete<'a>: SqlEmitter<'a, DeleteAst> {
    fn emit_delete(&mut self, ast: &'a DeleteAst, meta: &TableMetadata<'a>);
}
