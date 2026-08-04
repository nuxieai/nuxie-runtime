use crate::records::ast_stat_type_function::AstStatTypeFunction;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatTypeFunction {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_type_function(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_expr_visit(
                    self.body as *mut crate::records::ast_expr::AstExpr,
                    visitor,
                );
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_type_function_visit(this: *mut AstStatTypeFunction, visitor: *mut dyn AstVisitor) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
