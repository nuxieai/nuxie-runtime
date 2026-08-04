use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_group::AstExprGroup;
use luaur_ast::records::ast_expr_type_assertion::AstExprTypeAssertion;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::rtti::{ast_node_as, AstNodeClass};

pub fn unwrap_expr_of_type<T>(node: *mut AstExpr) -> *mut T
where
    T: AstNodeClass,
{
    if node.is_null() {
        return core::ptr::null_mut();
    }

    let expr = unsafe { ast_node_as::<T>(node as *mut AstNode) };
    if !expr.is_null() {
        return expr;
    }

    let group = unsafe { ast_node_as::<AstExprGroup>(node as *mut AstNode) };
    if !group.is_null() {
        return unwrap_expr_of_type::<T>(unsafe { (*group).expr });
    }

    let assertion = unsafe { ast_node_as::<AstExprTypeAssertion>(node as *mut AstNode) };
    if !assertion.is_null() {
        return unwrap_expr_of_type::<T>(unsafe { (*assertion).expr });
    }

    core::ptr::null_mut()
}
