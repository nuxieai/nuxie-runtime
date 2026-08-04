use crate::records::ast_expr_constant_bool::AstExprConstantBool;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstExprConstantBool {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_expr_constant_bool(self as *const Self as *mut core::ffi::c_void);
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_constant_bool_visit(this: *mut AstExprConstantBool, visitor: *mut dyn AstVisitor) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
