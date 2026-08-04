//! Faithful port of Luau `CstNode` — the base of every concrete-syntax-tree
//! node (`Ast/include/Luau/Cst.h`).
//!
//! The CST mirror of `AstNode`: `{ const int classIndex }` plus the
//! `is<T>()`/`as<T>()` RTTI helpers (here keyed on `CstClassIndex()`). Unlike
//! `AstNode` it carries no `Location` (trivia/offsets live in the concrete
//! nodes). `#[repr(C)]` with `base` first on each `CstX : CstNode` keeps the
//! base at offset 0 so the downcast in [`crate::rtti`] is a pointer cast.

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstNode {
    pub class_index: i32,
}
