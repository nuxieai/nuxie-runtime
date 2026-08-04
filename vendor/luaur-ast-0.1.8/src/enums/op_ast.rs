//! Node: `cxx:Enum:Luau.Ast:Ast/include/Luau/Ast.h:593:Op`
//!
//! `AstExprUnary::Op` (`Not` / `Minus` / `Len`). The nested C++ enum is
//! translated as the standalone `AstExprUnaryOp` living next to its record;
//! this node is the same enum, re-exported under its C++-local name `Op` so the
//! one definition stays the single source of truth (no drift between copies).

pub use crate::records::ast_expr_unary::AstExprUnaryOp as Op;
