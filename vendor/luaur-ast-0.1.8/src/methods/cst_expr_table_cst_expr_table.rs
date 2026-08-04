use crate::records::ast_array::AstArray;
use crate::records::cst_expr_table::{CstExprTable, CstExprTableItem};
use crate::records::cst_node::CstNode;
use crate::rtti::CstNodeClass;

impl CstExprTable {
    pub fn new(items: AstArray<CstExprTableItem>) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            items,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_expr_table_cst_expr_table(items: AstArray<CstExprTableItem>) -> CstExprTable {
    CstExprTable::new(items)
}
