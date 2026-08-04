//! `AstExprTable::Item` (`Ast/include/Luau/Ast.h`).
//!
//! The canonical definition (the `Item` struct + its `ItemKind` enum) lives in
//! the owner record `ast_expr_table.rs` (where `AstExprTable.items:
//! AstArray<Item>` uses it). This per-item file just re-exports it so the graph
//! node resolves to the same type — avoiding a duplicate, divergent `Item`.

pub use crate::records::ast_expr_table::{Item, ItemKind};
