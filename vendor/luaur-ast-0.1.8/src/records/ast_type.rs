//! Faithful port of Luau `AstType : AstNode` (`Ast/include/Luau/Ast.h`).
//!
//! The abstract base of every type-annotation node. It adds no fields over
//! `AstNode` (only the `as_type` override). `#[repr(C)]` with `base` first keeps
//! the `AstNode` subobject at offset 0 for the RTTI pointer casts in
//! [`crate::rtti`].

use crate::records::ast_node::AstNode;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstType {
    pub base: AstNode,
}
