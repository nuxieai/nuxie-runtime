use crate::records::ast_expr_varargs::AstExprVarargs;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstExprVarargs {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_expr_varargs(self as *const Self as *mut core::ffi::c_void);
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_varargs_visit(this: &AstExprVarargs, visitor: &mut dyn AstVisitor) {
    this.visit(visitor);
}
