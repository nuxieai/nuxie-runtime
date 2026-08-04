use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_local::AstLocal;

use crate::records::constant::Constant;

#[derive(Debug, Clone)]
pub struct InlineArg {
    pub(crate) local: *mut AstLocal,
    pub(crate) reg: u8,
    pub(crate) value: Constant,
    pub(crate) allocpc: u32,
    pub(crate) init: *mut AstExpr,
}

impl Default for InlineArg {
    fn default() -> Self {
        Self {
            local: core::ptr::null_mut(),
            reg: 0,
            value: Constant::default(),
            allocpc: 0,
            init: core::ptr::null_mut(),
        }
    }
}
