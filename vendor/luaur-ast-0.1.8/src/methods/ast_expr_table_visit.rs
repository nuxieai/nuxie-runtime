use crate::records::ast_expr_table::AstExprTable;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::{ast_expr_visit, AstVisitable};

impl crate::visit::AstVisitable for AstExprTable {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_expr_table(self as *const Self as *mut core::ffi::c_void) {
            for item in self.items.iter() {
                if !item.key.is_null() {
                    unsafe {
                        ast_expr_visit(item.key, visitor);
                    }
                }

                unsafe {
                    ast_expr_visit(item.value, visitor);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_table_visit(this: *const AstExprTable, visitor: &mut dyn AstVisitor) {
    unsafe {
        (*this).visit(visitor);
    }
}
