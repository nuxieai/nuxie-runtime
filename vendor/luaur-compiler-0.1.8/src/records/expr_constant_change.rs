use crate::records::constant::Constant;
use luaur_ast::records::ast_expr::AstExpr;

#[derive(Debug, Clone)]
pub struct ExprConstantChange {
    pub(crate) key: *mut AstExpr,
    pub(crate) old_value: Constant,
    pub(crate) was_absent: bool,
}

impl Default for ExprConstantChange {
    fn default() -> Self {
        Self {
            key: core::ptr::null_mut(),
            old_value: Constant::default(),
            was_absent: false,
        }
    }
}
