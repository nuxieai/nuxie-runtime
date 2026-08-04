use crate::records::constant::Constant;
use luaur_ast::records::ast_local::AstLocal;

#[derive(Debug, Clone)]
pub struct LocalConstantChange {
    pub(crate) key: *mut AstLocal,
    pub(crate) old_value: Constant,
    pub(crate) was_absent: bool,
}

impl Default for LocalConstantChange {
    fn default() -> Self {
        Self {
            key: core::ptr::null_mut(),
            old_value: Constant::default(),
            was_absent: false,
        }
    }
}
