use crate::records::ast_expr_group::AstExprGroup;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstExprGroup {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_expr_group(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_expr_visit(self.expr, visitor);
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_group_visit(this: *const AstExprGroup, visitor: &mut dyn AstVisitor) {
    unsafe {
        (*this).visit(visitor);
    }
}
