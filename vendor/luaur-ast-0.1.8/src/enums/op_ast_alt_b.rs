//! Node: `cxx:Enum:Luau.Ast:Ast/include/Luau/Ast.h:615:Op`
//!
//! `AstExprBinary::Op` (`Add` .. `Or`, plus the `Op__Count` sentinel). The
//! nested C++ enum is translated as the standalone `AstExprBinary_Op` living
//! next to its record; this node is the same enum, re-exported under its
//! C++-local name `Op` so the one definition stays the single source of truth.

pub use crate::records::ast_expr_binary::AstExprBinary_Op as Op;
