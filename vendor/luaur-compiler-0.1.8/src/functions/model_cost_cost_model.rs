use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::visit::ast_node_visit;
use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::records::constant::Constant;
use crate::records::cost_visitor::CostVisitor;

pub fn model_cost_ast_node_ast_local_usize_dense_hash_map_ast_expr_call_i32_dense_hash_map_ast_expr_constant(
    root: *mut AstNode,
    vars: *const *mut AstLocal,
    var_count: usize,
    builtins: &DenseHashMap<*mut AstExprCall, i32>,
    constants: &DenseHashMap<*mut AstExpr, Constant>,
) -> u64 {
    let mut visitor = CostVisitor::cost_visitor(builtins, constants);

    // C++ indexes `vars[i]` directly and never touches `vars` when varCount == 0.
    // `slice::from_raw_parts(null, 0)` is UB in Rust (pointer must be non-null even
    // for len 0), so index the raw pointer directly to match C++ semantics.
    let mut i = 0;
    while i < var_count && i < 7 {
        let var_ptr = unsafe { *vars.add(i) };
        *visitor.vars.get_or_insert(var_ptr) = 0xffu64 << (i * 8 + 8);
        i += 1;
    }

    unsafe {
        ast_node_visit(root, &mut visitor);
    }

    visitor.result.model
}
