use crate::records::cst_node::CstNode;
use crate::records::cst_stat_type_function::CstStatTypeFunction;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstStatTypeFunction {
    pub fn new(type_keyword_position: Position, function_keyword_position: Position) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            type_keyword_position,
            function_keyword_position,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_stat_type_function_cst_stat_type_function(
    type_keyword_position: Position,
    function_keyword_position: Position,
) -> CstStatTypeFunction {
    CstStatTypeFunction::new(type_keyword_position, function_keyword_position)
}
