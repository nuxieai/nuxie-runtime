use crate::records::ast_type_pack_generic::AstTypePackGeneric;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstTypePackGeneric {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_type_pack_generic(self as *const Self as *mut core::ffi::c_void);
    }
}
