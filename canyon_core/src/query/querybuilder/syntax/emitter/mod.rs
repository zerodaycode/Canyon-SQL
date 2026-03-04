pub(crate) mod backends;
mod types;

use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::ast::delete::DeleteAst;
use crate::query::querybuilder::syntax::ast::insert::InsertAst;
use crate::query::querybuilder::syntax::ast::update::UpdateAst;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::emitter::types::select::EmitSelect;
use crate::query::querybuilder::syntax::query_kind::QueryKind;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::SqlTokens;
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
pub trait SqlEmitter<'a>
where
    Self: 'a,
{
    /// The [`SqlDialect`] used by this emitter.
    ///
    /// Each backend emitter selects a dialect type that implements
    /// [`SqlDialect`]. This associated type must be distinct for each
    /// backend so that syntax differences can be encoded in the type
    /// system.
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
    fn emit(&mut self, ast: &impl AstProcessor<'a>, base_ast: &BaseAst<'a>) -> SqlTokens<'a>
    where
        Self: Sized,
    {
        // Default implementation delegates:
        match ast.query_kind() {
            QueryKind::Select => self.emit_select(ast, base_ast),
            QueryKind::Insert => self.emit_select(ast, base_ast),
            QueryKind::Update => self.emit_select(ast, base_ast),
            QueryKind::Delete => self.emit_select(ast, base_ast),
        }
    } // TODO: should emit as the outer wrapper really return the emitter internal buffer?
}

pub trait EmitInsert<'a>: SqlEmitter<'a> {
    fn emit_insert(&mut self, ast: &'a InsertAst<'a>, meta: &TableMetadata<'a>) -> SqlTokens<'a>;
}

pub trait EmitUpdate<'a>: SqlEmitter<'a> {
    fn emit_update(&mut self, ast: &'a UpdateAst<'a>, meta: &TableMetadata<'a>) -> SqlTokens<'a>;
}

pub trait EmitDelete<'a>: SqlEmitter<'a> {
    fn emit_delete(&mut self, ast: &'a DeleteAst, meta: &TableMetadata<'a>) -> SqlTokens<'a>;
}
