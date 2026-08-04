use crate::records::const_upvalue_visitor::ConstUpvalueVisitor;
use luaur_ast::records::ast_expr_local::AstExprLocal;

impl ConstUpvalueVisitor {
    pub fn visit_ast_expr_local(&mut self, node: *mut AstExprLocal) -> bool {
        unsafe {
            if (*node).upvalue && (*self.self_).is_constant(node as *mut _) {
                self.upvals.push((*node).local);
            }
        }
        false
    }
}
