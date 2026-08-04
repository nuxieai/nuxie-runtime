use crate::records::ast_expr::AstExpr;
use crate::records::ast_stat_assign::AstStatAssign;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::{ast_expr_visit, AstVisitable};

impl AstVisitable for AstStatAssign {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_assign(self as *const Self as *mut core::ffi::c_void) {
            for &lvalue in self.vars.iter() {
                unsafe {
                    ast_expr_visit(lvalue, visitor);
                }
            }

            for &expr in self.values.iter() {
                unsafe {
                    ast_expr_visit(expr, visitor);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_assign_visit(this: *const AstStatAssign, visitor: &mut dyn AstVisitor) {
    unsafe {
        (*this).visit(visitor);
    }
}
