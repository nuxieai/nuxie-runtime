use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::cst_stat_return::CstStatReturn;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstStatReturn {
    pub fn new(comma_positions: AstArray<Position>) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            comma_positions: comma_positions,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_stat_return_cst_stat_return(comma_positions: AstArray<Position>) -> CstStatReturn {
    CstStatReturn::new(comma_positions)
}
