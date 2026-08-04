use crate::records::ast_array::AstArray;
use crate::records::ast_type::AstType;

#[repr(C)]
#[derive(Debug)]
pub struct AstTypeUnion {
    pub base: AstType,
    pub types: AstArray<*mut AstType>,
}

impl crate::rtti::AstNodeClass for AstTypeUnion {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypeUnion");
}
