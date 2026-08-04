use crate::records::ast_stat_while::AstStatWhile;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::{ast_expr_visit, ast_stat_visit, AstVisitable};

impl AstVisitable for AstStatWhile {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_while(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                ast_expr_visit(self.condition, visitor);
                ast_stat_visit(self.body as *mut _, visitor);
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_while_visit(this: *const AstStatWhile, visitor: &mut dyn AstVisitor) {
    unsafe {
        (*this).visit(visitor);
    }
}
