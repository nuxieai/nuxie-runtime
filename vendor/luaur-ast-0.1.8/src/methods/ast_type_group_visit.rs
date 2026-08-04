use crate::records::ast_type_group::AstTypeGroup;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::{ast_type_visit, AstVisitable};

impl AstVisitable for AstTypeGroup {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_type_group(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                ast_type_visit(self.type_, visitor);
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_group_visit(this: &AstTypeGroup, visitor: &mut dyn AstVisitor) {
    this.visit(visitor);
}
