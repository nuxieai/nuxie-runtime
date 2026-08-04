use crate::records::ast_type::AstType;
use crate::records::ast_type_union::AstTypeUnion;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstTypeUnion {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_type_union(self as *const Self as *mut core::ffi::c_void) {
            for &type_ptr in self.types.iter() {
                unsafe {
                    crate::visit::ast_type_visit(type_ptr, visitor);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_union_visit(this: *const AstTypeUnion, visitor: &mut dyn AstVisitor) {
    unsafe {
        (*this).visit(visitor);
    }
}
