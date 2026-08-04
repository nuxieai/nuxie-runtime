use crate::records::ast_local::AstLocal;

#[derive(Debug, Clone, Copy)]
pub struct Local {
    pub local: *mut AstLocal,
    pub offset: u32,
}

impl Default for Local {
    fn default() -> Self {
        Self {
            local: core::ptr::null_mut(),
            offset: 0,
        }
    }
}
