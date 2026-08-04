use crate::records::ast_expr_instantiate::AstExprInstantiate;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstExprInstantiate {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_expr_instantiate(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_expr_visit(self.expr, visitor);
            }
            crate::functions::visit_type_or_pack_array::visit_type_or_pack_array(
                visitor,
                self.type_arguments,
            );
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_instantiate_visit(this: &AstExprInstantiate, visitor: &mut dyn AstVisitor) {
    this.visit(visitor);
}
