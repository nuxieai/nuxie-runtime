use crate::records::ast_type::AstType;
use crate::records::ast_type_pack::AstTypePack;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AstTypeOrPack {
    pub r#type: *mut AstType,
    pub type_pack: *mut AstTypePack,
}

impl Default for AstTypeOrPack {
    fn default() -> Self {
        Self {
            r#type: core::ptr::null_mut(),
            type_pack: core::ptr::null_mut(),
        }
    }
}
