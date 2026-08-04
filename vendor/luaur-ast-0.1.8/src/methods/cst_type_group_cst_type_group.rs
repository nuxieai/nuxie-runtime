use crate::records::cst_node::CstNode;
use crate::records::cst_type_group::CstTypeGroup;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstTypeGroup {
    pub fn new(close_position: Position) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            close_position,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_type_group_cst_type_group(close_position: Position) -> CstTypeGroup {
    CstTypeGroup::new(close_position)
}
