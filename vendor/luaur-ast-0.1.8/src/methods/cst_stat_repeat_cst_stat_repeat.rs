use crate::records::cst_node::CstNode;
use crate::records::cst_stat_repeat::CstStatRepeat;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstStatRepeat {
    #[allow(non_snake_case)]
    pub fn new(until_position: Position) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            until_position,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_stat_repeat_cst_stat_repeat(until_position: Position) -> CstStatRepeat {
    CstStatRepeat::new(until_position)
}
