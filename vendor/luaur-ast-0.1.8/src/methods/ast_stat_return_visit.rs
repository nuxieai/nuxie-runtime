use crate::records::ast_expr::AstExpr;
use crate::records::ast_stat_return::AstStatReturn;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::{ast_expr_visit, AstVisitable};

impl AstVisitable for AstStatReturn {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_return(self as *const Self as *mut core::ffi::c_void) {
            for &expr in self.list.iter() {
                unsafe {
                    ast_expr_visit(expr, visitor);
                }
            }
        }
    }
}
