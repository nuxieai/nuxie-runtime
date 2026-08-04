use crate::records::ast_stat_error::AstStatError;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatError {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_error(self as *const Self as *mut core::ffi::c_void) {
            for &expression in self.expressions.iter() {
                unsafe {
                    crate::visit::ast_expr_visit(expression, visitor);
                }
            }

            for &statement in self.statements.iter() {
                unsafe {
                    crate::visit::ast_stat_visit(statement, visitor);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_error_visit(this: &AstStatError, visitor: &mut dyn AstVisitor) {
    this.visit(visitor);
}
