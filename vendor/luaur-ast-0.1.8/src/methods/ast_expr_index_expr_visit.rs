use crate::records::ast_expr_index_expr::AstExprIndexExpr;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::{ast_expr_visit, AstVisitable};

impl AstVisitable for AstExprIndexExpr {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_expr_index_expr(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                ast_expr_visit(self.expr, visitor);
                ast_expr_visit(self.index, visitor);
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_index_expr_visit(this: *mut AstExprIndexExpr, visitor: *mut dyn AstVisitor) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
