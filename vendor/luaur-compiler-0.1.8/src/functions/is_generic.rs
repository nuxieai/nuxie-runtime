use luaur_ast::records::ast_array::AstArray;
use luaur_ast::records::ast_generic_type::AstGenericType;
use luaur_ast::records::ast_name::AstName;

pub fn is_generic(name: AstName, generics: &AstArray<*mut AstGenericType>) -> bool {
    for gt in generics.as_slice() {
        let gt_ref = unsafe { &**gt };
        if gt_ref.name.operator_eq_ast_name(&name) {
            return true;
        }
    }
    false
}
