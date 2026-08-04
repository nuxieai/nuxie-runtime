use crate::records::ast_expr_index_name::AstExprIndexName;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstExprIndexName {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_expr_index_name(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_expr_visit(self.expr, visitor);
            }
        }
    }
}
