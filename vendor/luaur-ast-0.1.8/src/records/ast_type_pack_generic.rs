use crate::records::ast_name::AstName;
use crate::records::ast_type_pack::AstTypePack;

#[repr(C)]
#[derive(Debug)]
pub struct AstTypePackGeneric {
    pub base: AstTypePack,
    pub generic_name: AstName,
}

impl crate::rtti::AstNodeClass for AstTypePackGeneric {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypePackGeneric");
}

#[allow(non_upper_case_globals)]
pub const AstTypePackGeneric_ClassIndex: i32 =
    <AstTypePackGeneric as crate::rtti::AstNodeClass>::CLASS_INDEX;
