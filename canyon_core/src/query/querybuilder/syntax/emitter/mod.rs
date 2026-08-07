pub(crate) mod backends;
pub(crate) mod types;

use crate::connection::database_type::DatabaseType;
use crate::query::querybuilder::syntax::{
    ast::BaseAst,
    dialect::SqlDialect,
    query_kind::QueryKind,
    tokens::SqlTokens,
};

#[cfg(feature = "postgres")]
use crate::query::querybuilder::syntax::emitter::backends::PgEmitter;

#[cfg(feature = "mssql")]
use crate::query::querybuilder::syntax::emitter::backends::SqlServerEmitter;

#[cfg(feature = "mysql")]
use crate::query::querybuilder::syntax::emitter::backends::MySqlEmitter;

// ---------- AST Processor marker trait ----------

pub trait AstProcessor<'a>: Default {
    fn query_kind(&self) -> QueryKind;
}

pub type EmitStep<'a, P> =
fn(&P, &mut BaseAst<'a>, &mut SqlTokens<'a>);

// ---------- Backend-specific conditional bounds ----------

mod backend_bounds {
    use super::{AstProcessor, BaseAst, SqlEmitter, SqlTokens};

    // -------------------------------------------------------------------------
    // PostgreSQL
    // -------------------------------------------------------------------------

    #[cfg(feature = "postgres")]
    use super::PgEmitter;

    #[cfg(feature = "postgres")]
    pub trait PostgresBackendEmittable<'a>: AstProcessor<'a> {
        fn emit_postgres(
            &self,
            base_ast: &mut BaseAst<'a>,
        ) -> SqlTokens<'a>;
    }

    #[cfg(feature = "postgres")]
    impl<'a, P> PostgresBackendEmittable<'a> for P
    where
        P: AstProcessor<'a> + 'a,
        PgEmitter: SqlEmitter<'a, P>,
    {
        #[inline]
        fn emit_postgres(
            &self,
            base_ast: &mut BaseAst<'a>,
        ) -> SqlTokens<'a> {
            PgEmitter::default().emit(self, base_ast)
        }
    }

    #[cfg(not(feature = "postgres"))]
    pub trait PostgresBackendEmittable<'a>: AstProcessor<'a> {}

    #[cfg(not(feature = "postgres"))]
    impl<'a, P> PostgresBackendEmittable<'a> for P
    where
        P: AstProcessor<'a> + 'a,
    {
    }

    // -------------------------------------------------------------------------
    // MySQL
    // -------------------------------------------------------------------------

    #[cfg(feature = "mysql")]
    use super::MySqlEmitter;

    #[cfg(feature = "mysql")]
    pub trait MySqlBackendEmittable<'a>: AstProcessor<'a> {
        fn emit_mysql(
            &self,
            base_ast: &mut BaseAst<'a>,
        ) -> SqlTokens<'a>;
    }

    #[cfg(feature = "mysql")]
    impl<'a, P> MySqlBackendEmittable<'a> for P
    where
        P: AstProcessor<'a> + 'a,
        MySqlEmitter: SqlEmitter<'a, P>,
    {
        #[inline]
        fn emit_mysql(
            &self,
            base_ast: &mut BaseAst<'a>,
        ) -> SqlTokens<'a> {
            MySqlEmitter::default().emit(self, base_ast)
        }
    }

    #[cfg(not(feature = "mysql"))]
    pub trait MySqlBackendEmittable<'a>: AstProcessor<'a> {}

    #[cfg(not(feature = "mysql"))]
    impl<'a, P> MySqlBackendEmittable<'a> for P
    where
        P: AstProcessor<'a> + 'a,
    {
    }

    // -------------------------------------------------------------------------
    // SQL Server
    // -------------------------------------------------------------------------

    #[cfg(feature = "mssql")]
    use super::SqlServerEmitter;

    #[cfg(feature = "mssql")]
    pub trait SqlServerBackendEmittable<'a>: AstProcessor<'a> {
        fn emit_sql_server(
            &self,
            base_ast: &mut BaseAst<'a>,
        ) -> SqlTokens<'a>;
    }

    #[cfg(feature = "mssql")]
    impl<'a, P> SqlServerBackendEmittable<'a> for P
    where
        P: AstProcessor<'a> + 'a,
        SqlServerEmitter: SqlEmitter<'a, P>,
    {
        #[inline]
        fn emit_sql_server(
            &self,
            base_ast: &mut BaseAst<'a>,
        ) -> SqlTokens<'a> {
            SqlServerEmitter::default().emit(self, base_ast)
        }
    }

    #[cfg(not(feature = "mssql"))]
    pub trait SqlServerBackendEmittable<'a>: AstProcessor<'a> {}

    #[cfg(not(feature = "mssql"))]
    impl<'a, P> SqlServerBackendEmittable<'a> for P
    where
        P: AstProcessor<'a> + 'a,
    {
    }
}

use backend_bounds::{
    MySqlBackendEmittable,
    PostgresBackendEmittable,
    SqlServerBackendEmittable,
};

// ---------- Runtime backend dispatch ----------

pub trait BackendEmittable<'a>: AstProcessor<'a> {
    fn emit_for(
        database_type: DatabaseType,
        ast: &Self,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a>;
}

impl<'a, P> BackendEmittable<'a> for P
where
    P: AstProcessor<'a>
    + PostgresBackendEmittable<'a>
    + MySqlBackendEmittable<'a>
    + SqlServerBackendEmittable<'a>
    + 'a,
{
    fn emit_for(
        database_type: DatabaseType,
        ast: &Self,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a> {
        match database_type {
            #[cfg(feature = "postgres")]
            DatabaseType::PostgreSql => ast.emit_postgres(base_ast),
            #[cfg(feature = "mssql")]
            DatabaseType::SqlServer => ast.emit_sql_server(base_ast),
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => ast.emit_mysql(base_ast),
        }
    }
}

// ---------- SQL emitter ----------

pub trait SqlEmitter<'a, P>
where
    Self: Sized,
    P: AstProcessor<'a> + 'a,
{
    type Dialect: SqlDialect;

    /// Ordered emission plan for this AST and backend combination.
    const PLAN: &'a [EmitStep<'a, P>];

    #[inline]
    fn emit(
        &mut self,
        ast: &P,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a> {
        let mut tokens = SqlTokens::default();

        for step in Self::PLAN {
            step(ast, base_ast, &mut tokens);
        }

        tokens
    }
}
