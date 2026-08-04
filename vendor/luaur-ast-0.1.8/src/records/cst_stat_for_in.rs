#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatForIn {
    pub base: crate::records::cst_node::CstNode,
    pub vars_annotation_colon_positions:
        crate::records::ast_array::AstArray<crate::records::position::Position>,
    pub vars_comma_positions:
        crate::records::ast_array::AstArray<crate::records::position::Position>,
    pub values_comma_positions:
        crate::records::ast_array::AstArray<crate::records::position::Position>,
}

impl crate::rtti::CstNodeClass for CstStatForIn {
    const CLASS_INDEX: i32 = crate::rtti::cst_rtti_index("CstStatForIn");
}
