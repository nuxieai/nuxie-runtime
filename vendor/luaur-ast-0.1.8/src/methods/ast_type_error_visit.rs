use crate::records::ast_type::AstType;
use crate::records::ast_type_error::AstTypeError;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstTypeError {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_type_error(self as *const Self as *mut core::ffi::c_void) {
            for type_ptr in self.types.iter() {
                unsafe {
                    crate::visit::ast_type_visit(*type_ptr, visitor);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_error_visit(this: *mut AstTypeError, visitor: *mut dyn AstVisitor) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
