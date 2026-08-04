#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstStatTypeAlias {
    pub base: crate::records::cst_node::CstNode,
    pub type_keyword_position: crate::records::position::Position,
    pub generics_open_position: crate::records::position::Position,
    pub generics_comma_positions:
        crate::records::ast_array::AstArray<crate::records::position::Position>,
    pub generics_close_position: crate::records::position::Position,
    pub equals_position: crate::records::position::Position,
}

impl crate::rtti::CstNodeClass for CstStatTypeAlias {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("CstStatTypeAlias");
}
