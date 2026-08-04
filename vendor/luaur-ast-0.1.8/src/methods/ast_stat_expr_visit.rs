use crate::records::ast_stat_expr::AstStatExpr;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatExpr {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_expr(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_expr_visit(self.expr, visitor);
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_expr_visit(this: *const AstStatExpr, visitor: &mut dyn AstVisitor) {
    unsafe {
        (*this).visit(visitor);
    }
}
