use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::SqlToken;

// default impl returns None; concrete ASTs override to return Some(&self as &dyn ...)
pub trait AsEmitKind<'a> {
    fn as_emit_kind(&self) -> Option<&dyn EmitKind<'a>> {
        None
    }
}
pub trait AsEmitFrom<'a> {
    fn as_emit_from(&self) -> Option<&dyn EmitFrom<'a>> {
        None
    }
}
pub trait AsEmitBody<'a> {
    fn as_emit_body(&self) -> Option<&dyn EmitBody<'a>> {
        None
    }
}
// pub trait AsEmitConditions<'a> {
//     fn as_emit_conditions(&self) -> Option<&dyn EmitConditions<'a>> { None }
// }

// The façade trait: ToSql groups phases.
// Require AsEmit* so we can call as_emit_* on any AST type P.
pub trait ToSql<'a>: AsEmitKind<'a> + AsEmitFrom<'a> + AsEmitBody<'a> {
    fn emit_all(
        &self,
        meta: &TableMetadata<'a>,
        //conditions: &[ConditionClause<'a>],
        out: &mut Vec<SqlToken<'a>>,
    ) {
        // KIND
        if let Some(k) = self.as_emit_kind() {
            k.emit_kind(out);
        }

        // FROM
        if let Some(f) = self.as_emit_from() {
            f.emit_from(meta, out);
        }

        // BODY
        if let Some(b) = self.as_emit_body() {
            b.emit_body(out);
        }

        // CONDITIONS (if AST wants to override condition emission)
        // if let Some(c) = self.as_emit_conditions() {
        //     c.emit_conditions(conditions, out);
        // } else {
        //     // fallback generic emission of base conditions (if base handles it)
        //     for cond in conditions {
        //         cond.to_tokens(out); // your ConditionClause -> SqlToken
        //     }
        // }
    }
}

// ---------- AST Processor marker trait ----------
pub trait AstProcessor: Default {
    // TODO: get base? as mut ref for convenience?
} // TODO: maybe this and the other one are visitor related?

// ---------- Emit traits ----------
pub trait EmitKind<'a> {
    fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>);
}
pub trait EmitFrom<'a> {
    fn emit_from(&self, meta: &TableMetadata<'a>, out: &mut Vec<SqlToken<'a>>);
}
pub trait EmitBody<'a> {
    fn emit_body(&self, out: &mut Vec<SqlToken<'a>>);
}

pub trait Emit
//
//
// // ---------- QueryEmitter enum ----------
// pub enum QueryEmitter<'a> { // Isn't this almost queryKind?
//     // Raw(BaseAst<'a>),
//     Select(SelectAst<'a>),
//     Insert(InsertAst<'a>),
//     Update(UpdateAst<'a>),
//     Delete(DeleteAst),
// }
//
// impl<'a> QueryEmitter<'a> {
//     /// façade: executes all phases in logical order: KIND -> FROM -> BODY -> CONDITIONS
//     pub fn emit_all_phases(
//         &self,
//         meta: &TableMetadata<'a>,
//         conditions: &[ConditionClause<'a>],
//         out: &mut Vec<SqlToken<'a>>
//     ) {
//         // 1. kind
//         self.emit_kind(out);
//         // 2. from (if any)
//         self.emit_from(meta, out);
//         // 3. body (set/values/insert columns...)
//         self.emit_body(out); // TODO: swap body and conditions
//         // 4. conditions (WHERE / AND / OR)
//         for cond in conditions {
//             // cond.to_tokens(out);
//         }
//     }
// }
//
// impl<'a> EmitKind<'a> for QueryEmitter<'a> {
//     fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>) {
//         match self {
//             Self::Select(ast) => ast.emit_kind(out),
//             Self::Insert(ast) => ast.emit_kind(out),
//             Self::Update(ast) => ast.emit_kind(out),
//             Self::Delete(ast) => ast.emit_kind(out),
//         }
//     }
// }
//
// impl<'a> EmitFrom<'a> for QueryEmitter<'a> {
//     fn emit_from(&self, meta: &TableMetadata<'a>, out: &mut Vec<SqlToken<'a>>) {
//         match self {
//             Self::Select(ast) => ast.emit_from(meta, out),
//             Self::Insert(ast) => ast.emit_from(meta, out),
//             Self::Update(_) => { },
//             Self::Delete(ast) => ast.emit_from(meta, out)
//         }
//     }
// }
// impl<'a> EmitBody<'a> for QueryEmitter<'a> {
//     fn emit_body(&self, out: &mut Vec<SqlToken<'a>>) {
//         match self {
//             Self::Select(ast) => ast.emit_body(out),
//             Self::Insert(ast) => ast.emit_body(out),
//             Self::Update(ast) => ast.emit_body(out),
//             Self::Delete(_) => { /* delete has no body */ }
//         }
//     }
// }
