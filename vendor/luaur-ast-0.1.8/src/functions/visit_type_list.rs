//! `visit_type_list` (`Ast/src/Ast.cpp`) — recurse a visitor over an `AstTypeList`
//! (its element types and the optional tail pack).

use crate::records::ast_type_list::AstTypeList;
use crate::records::ast_visitor::AstVisitor;

#[allow(non_snake_case)]
pub fn visit_type_list(visitor: &mut dyn AstVisitor, list: &AstTypeList) {
    for &ty in list.types.iter() {
        if !ty.is_null() {
            unsafe {
                crate::visit::ast_type_visit(ty, visitor);
            }
        }
    }

    if !list.tail_type.is_null() {
        unsafe {
            crate::visit::ast_type_pack_visit(list.tail_type, visitor);
        }
    }
}
