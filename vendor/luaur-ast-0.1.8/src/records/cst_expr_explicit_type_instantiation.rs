#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstExprExplicitTypeInstantiation {
    pub base: crate::records::cst_node::CstNode,
    pub instantiation: crate::records::cst_type_instantiation::CstTypeInstantiation,
}

impl crate::rtti::CstNodeClass for CstExprExplicitTypeInstantiation {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstExprExplicitTypeInstantiation");
}
