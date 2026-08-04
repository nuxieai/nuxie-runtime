use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_global::AstExprGlobal;
use crate::records::ast_expr_index_expr::AstExprIndexExpr;
use crate::records::ast_expr_index_name::AstExprIndexName;
use crate::records::ast_expr_local::AstExprLocal;
use crate::records::ast_node::AstNode;
use crate::rtti::{ast_node_as, ast_node_is};

pub fn is_expr_l_value(expr: *mut AstExpr) -> bool {
    if expr.is_null() {
        return false;
    }

    unsafe {
        let node = expr as *mut AstNode;

        let is_local = if ast_node_is::<AstExprLocal>(&*node) {
            if luaur_common::FFlag::LuauConst2.get() {
                let local_expr = ast_node_as::<AstExprLocal>(node);
                !(*local_expr).local.is_null() && !(*(*local_expr).local).is_const
            } else {
                true
            }
        } else {
            false
        };

        is_local
            || ast_node_is::<AstExprGlobal>(&*node)
            || ast_node_is::<AstExprIndexExpr>(&*node)
            || ast_node_is::<AstExprIndexName>(&*node)
    }
}
