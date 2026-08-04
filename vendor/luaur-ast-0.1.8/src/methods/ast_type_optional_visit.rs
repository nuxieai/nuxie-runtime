use crate::records::ast_type_optional::AstTypeOptional;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstTypeOptional {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_type_optional(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_type_visit(self.type_, visitor);
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_optional_visit(this: *const AstTypeOptional, visitor: &mut dyn AstVisitor) {
    unsafe {
        (*this).visit(visitor);
    }
}
