#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstTypeReference {
    pub base: crate::records::cst_node::CstNode,
    pub prefix_point_position: crate::records::position::Position,
    pub open_parameters_position: crate::records::position::Position,
    pub parameters_comma_positions:
        crate::records::ast_array::AstArray<crate::records::position::Position>,
    pub close_parameters_position: crate::records::position::Position,
}

impl crate::rtti::CstNodeClass for CstTypeReference {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstTypeReference");
}
