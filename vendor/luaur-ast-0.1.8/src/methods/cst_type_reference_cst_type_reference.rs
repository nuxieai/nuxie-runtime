use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::cst_type_reference::CstTypeReference;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstTypeReference {
    pub fn new(
        prefix_point_position: Position,
        open_parameters_position: Position,
        parameters_comma_positions: AstArray<Position>,
        close_parameters_position: Position,
    ) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            prefix_point_position,
            open_parameters_position,
            parameters_comma_positions,
            close_parameters_position,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_type_reference_cst_type_reference(
    prefix_point_position: Position,
    open_parameters_position: Position,
    parameters_comma_positions: AstArray<Position>,
    close_parameters_position: Position,
) -> CstTypeReference {
    CstTypeReference::new(
        prefix_point_position,
        open_parameters_position,
        parameters_comma_positions,
        close_parameters_position,
    )
}
