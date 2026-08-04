use crate::records::ast_stat_declare_global::AstStatDeclareGlobal;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatDeclareGlobal {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_declare_global(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_type_visit(self.type_, visitor);
            }
        }
    }
}
