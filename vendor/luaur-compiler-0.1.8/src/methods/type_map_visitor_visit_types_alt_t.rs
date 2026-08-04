use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_if_else::AstExprIfElse;
use luaur_ast::records::ast_type::AstType;

impl TypeMapVisitor<'_> {
    pub fn visit_ast_expr_if_else(&mut self, node: *mut AstExprIfElse) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let node_ref = &*node;

            luaur_ast::visit::ast_expr_visit(node_ref.condition, self);
            luaur_ast::visit::ast_expr_visit(node_ref.true_expr, self);
            luaur_ast::visit::ast_expr_visit(node_ref.false_expr, self);

            let true_type_ptr = self.resolved_exprs.find(&node_ref.true_expr);
            let true_bc_type_ptr = self.expr_types.find(&node_ref.true_expr);
            let false_bc_type_ptr = self.expr_types.find(&node_ref.false_expr);

            if let (Some(&true_type), Some(&true_bc_type), Some(&false_bc_type)) =
                (true_type_ptr, true_bc_type_ptr, false_bc_type_ptr)
            {
                // Optimistic check that both expressions are of the same kind, as AstType* cannot be compared
                if true_bc_type == false_bc_type {
                    self.record_resolved_type_ast_expr_ast_type(node as *mut AstExpr, true_type);
                }
            }
        }

        false
    }
}
