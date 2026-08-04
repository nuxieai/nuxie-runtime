use crate::records::ast_array::AstArray;
use crate::records::ast_type::AstType;
use crate::rtti::AstNodeClass;

#[repr(C)]
#[derive(Debug)]
pub struct AstTypeSingletonString {
    pub base: AstType,
    pub value: AstArray<core::ffi::c_char>,
}

impl AstNodeClass for AstTypeSingletonString {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypeSingletonString");
}
