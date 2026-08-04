use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;

#[repr(C)]
#[derive(Debug)]
pub struct AstTypeGroup {
    pub base: AstType,
    pub type_: *mut AstType,
}

impl crate::rtti::AstNodeClass for AstTypeGroup {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypeGroup");
}

#[allow(non_snake_case)]
impl AstTypeGroup {
    pub const fn LUAU_RTTI() {
        crate::macros::luau_rtti::LUAU_RTTI::<Self>()
    }
}
