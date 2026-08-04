use crate::records::cst_node::CstNode;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstAttr {
    pub base: CstNode,
    pub has_at: bool,
}

impl crate::rtti::CstNodeClass for CstAttr {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstAttr");
}
