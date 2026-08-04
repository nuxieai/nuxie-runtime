use crate::records::ast_stat_continue::AstStatContinue;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatContinue {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_stat_continue(self as *const Self as *mut core::ffi::c_void);
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_continue_visit(this: *mut AstStatContinue, visitor: *mut dyn AstVisitor) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
