use crate::records::shape_visitor::ShapeVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_index_expr::AstExprIndexExpr;
use luaur_ast::records::ast_expr_index_name::AstExprIndexName;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::rtti::ast_node_as;

impl<'a> ShapeVisitor<'a> {
    pub fn assign(&mut self, var: *mut AstExpr) {
        if var.is_null() {
            return;
        }

        let var_ptr = var as *mut AstNode;

        let index_name = unsafe { ast_node_as::<AstExprIndexName>(var_ptr) };
        if !index_name.is_null() {
            let expr = unsafe { (*index_name).expr };
            let index = unsafe { (*index_name).index };
            self.assign_field_ast_expr_ast_name(expr, index);
            return;
        }

        let index_expr = unsafe { ast_node_as::<AstExprIndexExpr>(var_ptr) };
        if !index_expr.is_null() {
            let expr = unsafe { (*index_expr).expr };
            let index = unsafe { (*index_expr).index };
            self.assign_field_ast_expr_ast_expr(expr, index);
        }
    }
}
