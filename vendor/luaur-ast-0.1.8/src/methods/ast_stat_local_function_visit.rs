use crate::records::ast_stat_local_function::AstStatLocalFunction;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatLocalFunction {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_local_function(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_expr_visit(
                    self.func as *mut crate::records::ast_expr::AstExpr,
                    visitor,
                );
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_local_function_visit(
    this: *mut AstStatLocalFunction,
    visitor: *mut dyn AstVisitor,
) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
