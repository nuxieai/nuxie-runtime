use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::functions::model_cost_cost_model::model_cost_ast_node_ast_local_usize_dense_hash_map_ast_expr_call_i32_dense_hash_map_ast_expr_constant;
use crate::records::constant::Constant;

pub fn model_cost_ast_node_ast_local_usize(
    root: *mut AstNode,
    vars: *const *mut AstLocal,
    var_count: usize,
) -> u64 {
    let builtins = DenseHashMap::new(core::ptr::null_mut::<AstExprCall>());
    let constants = DenseHashMap::new(core::ptr::null_mut::<AstExpr>());

    model_cost_ast_node_ast_local_usize_dense_hash_map_ast_expr_call_i32_dense_hash_map_ast_expr_constant(
        root, vars, var_count, &builtins, &constants,
    )
}
