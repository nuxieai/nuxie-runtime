use crate::records::ast_stat_repeat::AstStatRepeat;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::{ast_expr_visit, ast_stat_visit, AstVisitable};

impl AstVisitable for AstStatRepeat {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_repeat(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                ast_stat_visit(self.body as *mut _, visitor);
                ast_expr_visit(self.condition, visitor);
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_repeat_visit(this: *mut AstStatRepeat, visitor: *mut dyn AstVisitor) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
