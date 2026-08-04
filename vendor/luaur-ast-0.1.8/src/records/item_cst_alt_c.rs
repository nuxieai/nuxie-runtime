//! `CstTypeTable::Item` (`Ast/include/Luau/Cst.h:425`).
//!
//! The canonical definition (the `Item` struct + its `Kind` enum) lives in the
//! owner record `cst_type_table.rs`. This per-item file re-exports it so the
//! graph node resolves to the same type (no duplicate, divergent `Item`). Note
//! this is a *different* `Item` from `item_cst.rs` (which is
//! `CstExprTable::Item`).

pub use crate::records::cst_type_table::{
    CstTypeTable_Item as Item, CstTypeTable_Item_Kind as Kind,
};
