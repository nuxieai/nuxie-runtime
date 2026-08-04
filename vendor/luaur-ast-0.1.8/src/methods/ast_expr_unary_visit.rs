use crate::records::ast_expr_unary::AstExprUnary;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstExprUnary {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_expr_unary(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_expr_visit(self.expr, visitor);
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_unary_visit(this: *mut AstExprUnary, visitor: &mut dyn AstVisitor) {
    unsafe {
        (*this).visit(visitor);
    }
}
