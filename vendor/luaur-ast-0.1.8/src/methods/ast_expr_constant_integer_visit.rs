use crate::records::ast_expr_constant_integer::AstExprConstantInteger;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstExprConstantInteger {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_expr_constant_integer(self as *const Self as *mut core::ffi::c_void);
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_constant_integer_visit(
    this: *mut AstExprConstantInteger,
    visitor: *mut dyn AstVisitor,
) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
