use crate::records::cst_node::CstNode;
use crate::records::cst_stat_local_function::CstStatLocalFunction;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstStatLocalFunction {
    pub fn new(local_keyword_position: Position, function_keyword_position: Position) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            local_keyword_position,
            function_keyword_position,
        }
    }
}
