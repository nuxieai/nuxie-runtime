use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstExprFunction {
    pub base: CstNode,
    pub function_keyword_position: Position,
    pub open_generics_position: Position,
    pub generics_comma_positions: AstArray<Position>,
    pub close_generics_position: Position,
    pub args_annotation_colon_positions: AstArray<Position>,
    pub args_comma_positions: AstArray<Position>,
    pub vararg_annotation_colon_position: Position,
    pub return_specifier_position: Position,
}

impl crate::rtti::CstNodeClass for CstExprFunction {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstExprFunction");
}
