use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr_group::AstExprGroup;
use luaur_ast::visit;

impl TypeMapVisitor<'_> {
    pub fn visit_ast_expr_group(&mut self, node: *mut AstExprGroup) -> bool {
        unsafe {
            let expr = (*node).expr;

            visit::ast_expr_visit(expr, self);

            if let Some(&ty_ptr) = self.resolved_exprs.find(&expr) {
                self.record_resolved_type_ast_expr_ast_type(node as *mut _, ty_ptr);
            }
        }

        false
    }
}
