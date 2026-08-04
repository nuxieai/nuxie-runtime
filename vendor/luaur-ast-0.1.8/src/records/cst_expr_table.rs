use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum CstExprTableSeparator {
    Comma,
    Semicolon,
    Missing,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CstExprTableItem {
    pub indexer_open_position: Position,
    pub indexer_close_position: Position,
    pub equals_position: Position,
    pub separator: CstExprTableSeparator,
    pub separator_position: Position,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstExprTable {
    pub base: CstNode,
    pub items: AstArray<CstExprTableItem>,
}

impl crate::rtti::CstNodeClass for CstExprTable {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstExprTable");
}

// C++ nested `CstExprTable::Separator` / `CstExprTable::Item` (Rust has no
// inherent associated types — these live at module scope).
pub type Separator = CstExprTableSeparator;
pub type Item = CstExprTableItem;

#[allow(non_snake_case)]
impl CstExprTable {
    pub const Comma: CstExprTableSeparator = CstExprTableSeparator::Comma;
    pub const Semicolon: CstExprTableSeparator = CstExprTableSeparator::Semicolon;
    pub const Missing: CstExprTableSeparator = CstExprTableSeparator::Missing;
}
