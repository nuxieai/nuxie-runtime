use crate::records::ast_stat_compound_assign::AstStatCompoundAssign;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatCompoundAssign {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_compound_assign(self as *const Self as *mut core::ffi::c_void) {
            unsafe {
                crate::visit::ast_expr_visit(self.var, visitor);
                crate::visit::ast_expr_visit(self.value, visitor);
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_compound_assign_visit(
    this: *mut AstStatCompoundAssign,
    visitor: *mut dyn AstVisitor,
) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
