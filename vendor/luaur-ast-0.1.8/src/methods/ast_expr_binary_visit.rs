use crate::records::ast_expr_binary::AstExprBinary;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::{ast_expr_visit, AstVisitable};

impl AstVisitable for AstExprBinary {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_expr_binary(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                ast_expr_visit(self.left, visitor);
                ast_expr_visit(self.right, visitor);
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_binary_visit(this: *const AstExprBinary, visitor: &mut dyn AstVisitor) {
    unsafe {
        (*this).visit(visitor);
    }
}
