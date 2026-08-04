use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::cst_type_table::{CstTypeTable, CstTypeTable_Item};
use crate::rtti::CstNodeClass;

impl CstTypeTable {
    pub fn new(items: AstArray<CstTypeTable_Item>, is_array: bool) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            items,
            is_array: is_array,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_type_table_cst_type_table(
    items: AstArray<CstTypeTable_Item>,
    is_array: bool,
) -> CstTypeTable {
    CstTypeTable::new(items, is_array)
}
