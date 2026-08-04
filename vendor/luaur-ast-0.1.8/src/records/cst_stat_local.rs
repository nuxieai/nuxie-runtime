use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatLocal {
    pub base: CstNode,
    pub declaration_keyword_position: Position,
    pub vars_annotation_colon_positions: AstArray<Position>,
    pub vars_comma_positions: AstArray<Position>,
    pub values_comma_positions: AstArray<Position>,
}

impl crate::rtti::CstNodeClass for CstStatLocal {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstStatLocal");
}
