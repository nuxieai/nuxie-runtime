use crate::records::ast_expr_constant_number::AstExprConstantNumber;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstExprConstantNumber {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_expr_constant_number(self as *const Self as *mut core::ffi::c_void);
    }
}
