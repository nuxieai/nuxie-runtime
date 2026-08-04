use crate::records::ast_expr_constant_nil::AstExprConstantNil;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstExprConstantNil {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_expr_constant_nil(self as *const Self as *mut core::ffi::c_void);
    }
}
