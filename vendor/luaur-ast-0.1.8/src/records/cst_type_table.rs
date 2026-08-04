use crate::records::ast_array::AstArray;
use crate::records::cst_expr_constant_string::CstExprConstantString;
use crate::records::cst_expr_table::Separator;
use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstTypeTable {
    pub base: CstNode,
    pub items: AstArray<CstTypeTable_Item>,
    pub is_array: bool,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstTypeTable_Item {
    pub kind: CstTypeTable_Item_Kind,
    pub indexer_open_position: Position,
    pub indexer_close_position: Position,
    pub colon_position: Position,
    pub separator: Separator,
    pub separator_position: Position,
    pub string_info: *mut CstExprConstantString,
    pub string_position: Position,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum CstTypeTable_Item_Kind {
    Indexer,
    Property,
    StringProperty,
}

impl crate::rtti::CstNodeClass for CstTypeTable {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstTypeTable");
}

// C++ nested `CstTypeTable::Item` / `CstTypeTable::Item::Kind` (no inherent
// associated types in Rust — aliases live at module scope).
pub type Item = CstTypeTable_Item;
pub type Kind = CstTypeTable_Item_Kind;

#[allow(non_snake_case)]
impl CstTypeTable_Item {
    pub const Indexer: CstTypeTable_Item_Kind = CstTypeTable_Item_Kind::Indexer;
    pub const Property: CstTypeTable_Item_Kind = CstTypeTable_Item_Kind::Property;
    pub const StringProperty: CstTypeTable_Item_Kind = CstTypeTable_Item_Kind::StringProperty;
}
