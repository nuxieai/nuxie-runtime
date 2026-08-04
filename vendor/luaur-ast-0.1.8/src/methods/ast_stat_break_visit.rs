use crate::records::ast_stat_break::AstStatBreak;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatBreak {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_stat_break(self as *const Self as *mut core::ffi::c_void);
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_break_visit(this: *mut AstStatBreak, visitor: *mut dyn AstVisitor) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
