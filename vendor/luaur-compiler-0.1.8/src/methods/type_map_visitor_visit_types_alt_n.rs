use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr_type_assertion::AstExprTypeAssertion;
use luaur_ast::visit;

impl TypeMapVisitor<'_> {
    pub fn visit_ast_expr_type_assertion(&mut self, node: *mut AstExprTypeAssertion) -> bool {
        unsafe {
            let node_ref = &*node;

            visit::ast_expr_visit(node_ref.expr, self);

            self.record_resolved_type_ast_expr_ast_type(
                node as *mut luaur_ast::records::ast_expr::AstExpr,
                node_ref.annotation as *const _,
            );
        }

        false
    }
}
