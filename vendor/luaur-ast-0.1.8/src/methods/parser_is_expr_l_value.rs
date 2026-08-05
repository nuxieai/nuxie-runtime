use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_global::AstExprGlobal;
use crate::records::ast_expr_index_expr::AstExprIndexExpr;
use crate::records::ast_expr_index_name::AstExprIndexName;
use crate::records::ast_expr_local::AstExprLocal;
use crate::records::ast_node::AstNode;
use crate::records::parser::Parser;
use crate::rtti::{ast_node_as, ast_node_is};

impl Parser {
    pub fn is_expr_l_value(&self, expr: *mut AstExpr) -> bool {
        if expr.is_null() {
            return false;
        }
        unsafe {
            let node = expr as *mut AstNode;
            let is_local = if ast_node_is::<AstExprLocal>(&*node) {
                let local_expr = ast_node_as::<AstExprLocal>(node);
                !(*local_expr).local.is_null() && !(*(*local_expr).local).is_const
            } else {
                false
            };
            let is_global = ast_node_is::<AstExprGlobal>(&*node)
                && !(luaur_common::FFlag::DebugLuauUserDefinedClasses.get()
                    && !self.get_matching_class(expr).is_null());
            is_local
                || is_global
                || ast_node_is::<AstExprIndexExpr>(&*node)
                || ast_node_is::<AstExprIndexName>(&*node)
        }
    }
}
