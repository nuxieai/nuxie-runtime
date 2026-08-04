use crate::records::ast_array::AstArray;
use crate::records::ast_type_or_pack::AstTypeOrPack;
use crate::records::ast_visitor::AstVisitor;

pub(crate) fn visit_type_or_pack_array(
    visitor: &mut dyn AstVisitor,
    array_of_type_or_pack: AstArray<AstTypeOrPack>,
) {
    for param in array_of_type_or_pack.as_slice() {
        if !param.r#type.is_null() {
            unsafe {
                crate::visit::ast_type_visit(param.r#type, visitor);
            }
        } else if !param.type_pack.is_null() {
            unsafe {
                crate::visit::ast_type_pack_visit(param.type_pack, visitor);
            }
        }
    }
}
