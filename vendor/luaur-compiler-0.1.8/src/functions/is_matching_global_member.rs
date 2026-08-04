use core::ffi::c_char;

use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_ast::records::ast_expr_index_name::AstExprIndexName;
use luaur_ast::records::ast_name::AstName;
use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::enums::global::Global;
use crate::functions::get_global_state::get_global_state;

pub fn is_matching_global_member(
    globals: &DenseHashMap<AstName, Global>,
    expr: *mut AstExprIndexName,
    library: *const c_char,
    member: *const c_char,
) -> bool {
    let expr_global = unsafe {
        let expr_ptr = expr;
        let obj = (*expr_ptr).expr;
        luaur_ast::rtti::ast_node_as::<AstExprGlobal>(
            obj as *mut luaur_ast::records::ast_node::AstNode,
        )
    };

    if !expr_global.is_null() {
        let object = unsafe { &*expr_global };
        return get_global_state(globals, object.name) == Global::Default
            && object.name.operator_eq_c_char(library)
            && unsafe { (*expr).index.operator_eq_c_char(member) };
    }

    false
}
