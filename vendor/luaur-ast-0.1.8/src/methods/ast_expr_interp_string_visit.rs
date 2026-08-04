use crate::records::ast_expr_interp_string::AstExprInterpString;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstExprInterpString {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_expr_interp_string(self as *const Self as *mut core::ffi::c_void) {
            for &expr in self.expressions.iter() {
                unsafe {
                    crate::visit::ast_expr_visit(expr, visitor);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_interp_string_visit(this: *mut AstExprInterpString, visitor: *mut dyn AstVisitor) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
