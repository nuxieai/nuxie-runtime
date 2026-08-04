use crate::records::fenv_visitor::FenvVisitor;
use core::ffi::CStr;
use luaur_ast::records::ast_expr_global::AstExprGlobal;

impl FenvVisitor<'_> {
    pub fn visit(&mut self, node: &AstExprGlobal) -> bool {
        unsafe {
            if !node.name.value.is_null() {
                let name_bytes = CStr::from_ptr(node.name.value).to_bytes();
                if name_bytes == b"getfenv" {
                    *self.getfenv_used = true;
                }
                if name_bytes == b"setfenv" {
                    *self.setfenv_used = true;
                }
            }
        }
        false
    }
}
