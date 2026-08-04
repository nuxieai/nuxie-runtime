use crate::records::ast_stat_declare_extern_type::AstStatDeclareExternType;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatDeclareExternType {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_declare_extern_type(self as *const Self as *mut core::ffi::c_void) {
            for prop in self.props.iter() {
                unsafe {
                    crate::visit::ast_type_visit(prop.ty, visitor);
                }
            }
        }
    }
}
