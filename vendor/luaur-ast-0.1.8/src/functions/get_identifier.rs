use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_global::AstExprGlobal;
use crate::records::ast_expr_local::AstExprLocal;
use crate::records::ast_name::AstName;
use crate::records::ast_node::AstNode;

#[allow(non_snake_case)]
pub fn get_identifier(node: *mut AstExpr) -> AstName {
    if node.is_null() {
        return AstName {
            value: core::ptr::null(),
        };
    }

    unsafe {
        let node_base = node as *mut AstNode;

        let global = crate::rtti::ast_node_as::<AstExprGlobal>(node_base);
        if !global.is_null() {
            return (*global).name;
        }

        let local = crate::rtti::ast_node_as::<AstExprLocal>(node_base);
        if !local.is_null() && !(*local).local.is_null() {
            return (*(*local).local).name;
        }
    }

    AstName {
        value: core::ptr::null(),
    }
}
