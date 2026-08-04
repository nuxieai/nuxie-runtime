use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_global::AstExprGlobal;
use crate::records::ast_expr_index_expr::AstExprIndexExpr;
use crate::records::ast_expr_index_name::AstExprIndexName;
use crate::records::ast_expr_local::AstExprLocal;
use crate::records::ast_node::AstNode;
use crate::rtti::ast_node_is;

#[allow(non_snake_case)]
pub fn is_l_value(expr: *const AstExpr) -> bool {
    if expr.is_null() {
        return false;
    }

    unsafe {
        let node = expr as *const AstNode;
        ast_node_is::<AstExprLocal>(&*node)
            || ast_node_is::<AstExprGlobal>(&*node)
            || ast_node_is::<AstExprIndexName>(&*node)
            || ast_node_is::<AstExprIndexExpr>(&*node)
    }
}
