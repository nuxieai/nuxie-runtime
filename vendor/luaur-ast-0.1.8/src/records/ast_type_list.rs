use crate::records::ast_array::AstArray;
use crate::records::ast_type::AstType;
use crate::records::ast_type_pack::AstTypePack;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub struct AstTypeList {
    pub types: AstArray<*mut AstType>,
    pub tail_type: *mut AstTypePack,
}

impl Default for AstTypeList {
    fn default() -> Self {
        Self {
            types: AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            },
            tail_type: core::ptr::null_mut(),
        }
    }
}
