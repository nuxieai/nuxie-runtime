use crate::records::undefined_local_visitor::UndefinedLocalVisitor;
use luaur_ast::records::ast_expr_local::AstExprLocal;

impl UndefinedLocalVisitor {
    pub fn visit_ast_expr_local(&mut self, node: *mut AstExprLocal) -> bool {
        unsafe {
            if !(*node).upvalue {
                self.check((*node).local);
            }
        }
        false
    }
}
