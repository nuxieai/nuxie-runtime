//! Faithful port of Luau `AstExpr : AstNode` (`Ast/include/Luau/Ast.h`).
//!
//! The abstract base of every expression node. It adds no fields over `AstNode`
//! (only the `as_expr` override). `#[repr(C)]` with `base` first keeps the
//! `AstNode` subobject at offset 0 so the RTTI pointer casts in [`crate::rtti`]
//! are sound.

use crate::records::ast_node::AstNode;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstExpr {
    pub base: AstNode,
}
