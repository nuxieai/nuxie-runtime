use crate::records::ast_type_singleton_string::AstTypeSingletonString;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstTypeSingletonString {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_type_singleton_string(self as *const Self as *mut core::ffi::c_void);
    }
}
