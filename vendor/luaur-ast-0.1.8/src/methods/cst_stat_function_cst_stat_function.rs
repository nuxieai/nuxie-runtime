use crate::records::cst_node::CstNode;
use crate::records::cst_stat_function::CstStatFunction;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstStatFunction {
    pub fn new(function_keyword_position: Position) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            function_keyword_position: function_keyword_position,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_stat_function_cst_stat_function(function_keyword_position: Position) -> CstStatFunction {
    CstStatFunction::new(function_keyword_position)
}
