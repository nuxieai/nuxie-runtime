use crate::records::ast_attr::AstAttr;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstAttr {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_attr(self as *const Self as *mut core::ffi::c_void);
    }
}

#[allow(non_snake_case)]
pub fn ast_attr_visit(node: *mut AstAttr, visitor: &mut dyn AstVisitor) {
    unsafe {
        (*node).visit(visitor);
    }
}
