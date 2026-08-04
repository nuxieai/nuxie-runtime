use crate::records::ast_type_pack_explicit::AstTypePackExplicit;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::{ast_type_pack_visit, ast_type_visit, AstVisitable};

impl AstVisitable for AstTypePackExplicit {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_type_pack_explicit(self as *const Self as *mut core::ffi::c_void) {
            for &type_ptr in self.type_list.types.iter() {
                unsafe {
                    ast_type_visit(type_ptr, visitor);
                }
            }

            if !self.type_list.tail_type.is_null() {
                unsafe {
                    ast_type_pack_visit(self.type_list.tail_type, visitor);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_pack_explicit_visit(this: *mut AstTypePackExplicit, visitor: *mut dyn AstVisitor) {
    unsafe {
        (*this).visit(&mut *visitor);
    }
}
