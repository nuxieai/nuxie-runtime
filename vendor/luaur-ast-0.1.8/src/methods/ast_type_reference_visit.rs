use crate::functions::visit_type_or_pack_array::visit_type_or_pack_array;
use crate::records::ast_type_reference::AstTypeReference;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstTypeReference {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_type_reference(self as *const Self as *mut core::ffi::c_void) {
            visit_type_or_pack_array(visitor, self.parameters);
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_reference_visit(this: &AstTypeReference, visitor: &mut dyn AstVisitor) {
    this.visit(visitor);
}
