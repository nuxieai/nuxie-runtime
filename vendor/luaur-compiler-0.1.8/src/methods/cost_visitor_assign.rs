use crate::records::cost_visitor::CostVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_local::AstExprLocal;

impl CostVisitor {
    pub(crate) fn assign(&mut self, expr: *mut AstExpr) {
        cost_visitor_assign(self, expr);
    }
}

#[allow(non_snake_case)]
pub fn cost_visitor_assign(visitor: &mut CostVisitor, expr: *mut AstExpr) {
    unsafe {
        if expr.is_null() {
            return;
        }

        let expr_local = luaur_ast::rtti::ast_node_as::<AstExprLocal>(
            expr as *mut luaur_ast::records::ast_node::AstNode,
        );
        if expr_local.is_null() {
            return;
        }

        let local = (*expr_local).local;
        if local.is_null() {
            return;
        }

        let key = &local;
        if let Some(found) = visitor.vars.find_mut(key) {
            *found = 0;
        }
    }
}

#[allow(non_snake_case)]
pub fn cost_visitor_assign_impl(visitor: &mut CostVisitor, expr: *mut AstExpr) {
    cost_visitor_assign(visitor, expr);
}
