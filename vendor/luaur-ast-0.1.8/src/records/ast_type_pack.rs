//! Faithful port of Luau `AstTypePack : AstNode` (`Ast/include/Luau/Ast.h`).
//!
//! The abstract base of every type-pack node (`AstTypePackExplicit`,
//! `AstTypePackVariadic`, `AstTypePackGeneric`). It adds no fields over
//! `AstNode`. `#[repr(C)]` with `base` first keeps the `AstNode` subobject at
//! offset 0 for the RTTI pointer casts in [`crate::rtti`].

use crate::records::ast_node::AstNode;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstTypePack {
    pub base: AstNode,
}
