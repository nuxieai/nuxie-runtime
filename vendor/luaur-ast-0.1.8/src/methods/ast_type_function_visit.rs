use crate::records::ast_type_function::AstTypeFunction;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstTypeFunction {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_type_function(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::functions::visit_type_list::visit_type_list(visitor, &self.arg_types);

                if !self.return_types.is_null() {
                    crate::visit::ast_type_pack_visit(self.return_types, visitor);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_function_visit(this: &AstTypeFunction, visitor: &mut dyn AstVisitor) {
    this.visit(visitor);
}
