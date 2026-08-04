use crate::records::ast_stat_declare_function::AstStatDeclareFunction;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatDeclareFunction {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_declare_function(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::functions::visit_type_list::visit_type_list(visitor, &self.params);

                if !self.ret_types.is_null() {
                    crate::visit::ast_type_pack_visit(self.ret_types, visitor);
                }
            }
        }
    }
}
