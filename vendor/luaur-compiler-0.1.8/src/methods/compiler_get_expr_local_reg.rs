use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;

impl Compiler {
    pub fn get_expr_local_reg(&mut self, node: *mut AstExpr) -> i32 {
        unsafe {
            let expr = self.get_expr_local(node);
            if expr.is_null() {
                return -1;
            }

            match self.locals.find(&(*expr).local) {
                Some(l) if l.allocated => l.reg as i32,
                _ => -1,
            }
        }
    }
}
