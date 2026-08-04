use core::ffi::c_char;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::rtti;
use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::enums::global::Global;
use crate::functions::get_global_state::get_global_state;

pub fn is_matching_global(
    globals: &DenseHashMap<AstName, Global>,
    node: *mut AstExpr,
    name: *const c_char,
) -> bool {
    let expr_global = unsafe { rtti::ast_node_as::<AstExprGlobal>(node as *mut AstNode) };

    if !expr_global.is_null() {
        let expr = unsafe { &*expr_global };
        return get_global_state(globals, expr.name) == Global::Default
            && expr.name.operator_eq_c_char(name);
    }

    false
}
