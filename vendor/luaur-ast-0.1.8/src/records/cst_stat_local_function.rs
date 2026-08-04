#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatLocalFunction {
    pub base: crate::records::cst_node::CstNode,
    pub attr_lists: crate::records::ast_array::AstArray<*mut crate::records::cst_attr_list::CstAttrList>,
    pub local_keyword_position: crate::records::position::Position,
    pub function_keyword_position: crate::records::position::Position,
}

impl crate::rtti::CstNodeClass for CstStatLocalFunction {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstStatLocalFunction");
}
