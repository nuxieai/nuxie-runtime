use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_error::AstExprError;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstExprError {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_expr_error(self as *const Self as *mut core::ffi::c_void) {
            for i in 0..self.expressions.size {
                unsafe {
                    let expression = *self.expressions.data.add(i);
                    crate::visit::ast_expr_visit(expression, visitor);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_error_visit(this: &AstExprError, visitor: &mut dyn AstVisitor) {
    this.visit(visitor);
}
