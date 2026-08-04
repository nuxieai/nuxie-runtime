use crate::records::ast_stat_function::AstStatFunction;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatFunction {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_function(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_expr_visit(self.name, visitor);
                crate::visit::ast_expr_visit(
                    self.func as *mut crate::records::ast_expr::AstExpr,
                    visitor,
                );
            }
        }
    }
}
