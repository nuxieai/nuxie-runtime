use crate::records::ast_stat_if::AstStatIf;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatIf {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_if(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_expr_visit(self.condition, visitor);
                crate::visit::ast_stat_visit(self.thenbody as *mut _, visitor);

                if !self.elsebody.is_null() {
                    crate::visit::ast_stat_visit(self.elsebody, visitor);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_if_visit(this: *mut AstStatIf, visitor: *mut dyn AstVisitor) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
