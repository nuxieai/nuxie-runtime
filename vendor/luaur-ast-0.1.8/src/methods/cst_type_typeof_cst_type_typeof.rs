use crate::records::cst_node::CstNode;
use crate::records::cst_type_typeof::CstTypeTypeof;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstTypeTypeof {
    pub fn new(open_position: Position, close_position: Position) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            open_position,
            close_position,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_type_typeof_cst_type_typeof(
    open_position: Position,
    close_position: Position,
) -> CstTypeTypeof {
    CstTypeTypeof::new(open_position, close_position)
}
