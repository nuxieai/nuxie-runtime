use crate::records::ast_array::AstArray;
use crate::records::ast_type::AstType;
use crate::rtti::AstNodeClass;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstTypeError {
    pub base: AstType,
    pub types: AstArray<*mut AstType>,
    pub is_missing: bool,
    pub message_index: u32,
}

impl AstNodeClass for AstTypeError {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypeError");
}
