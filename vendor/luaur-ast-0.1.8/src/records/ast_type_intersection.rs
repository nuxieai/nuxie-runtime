use crate::records::ast_array::AstArray;
use crate::records::ast_type::AstType;

#[repr(C)]
#[derive(Debug)]
pub struct AstTypeIntersection {
    pub base: AstType,
    pub types: AstArray<*mut AstType>,
}

impl crate::rtti::AstNodeClass for AstTypeIntersection {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypeIntersection");
}
