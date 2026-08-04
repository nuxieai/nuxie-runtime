use crate::records::ast_expr_local::AstExprLocal;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstExprLocal {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_expr_local(self as *const Self as *mut core::ffi::c_void);
    }
}
