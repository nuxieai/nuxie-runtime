use crate::records::ast_type_typeof::AstTypeTypeof;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstTypeTypeof {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_type_typeof(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_expr_visit(self.expr, visitor);
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_typeof_visit(this: &AstTypeTypeof, visitor: &mut dyn AstVisitor) {
    this.visit(visitor);
}
