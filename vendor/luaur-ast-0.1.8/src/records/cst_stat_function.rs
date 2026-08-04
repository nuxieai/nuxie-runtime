use crate::records::ast_array::AstArray;
use crate::records::cst_attr_list::CstAttrList;
use crate::records::cst_node::CstNode;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatFunction {
    pub base: CstNode,
    pub attr_lists: AstArray<*mut CstAttrList>,
    pub function_keyword_position: Position,
}

impl crate::rtti::CstNodeClass for CstStatFunction {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstStatFunction");
}
