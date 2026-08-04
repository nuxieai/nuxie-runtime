use crate::records::type_map_visitor::TypeMapVisitor;

use luaur_ast::records::ast_expr_index_expr::AstExprIndexExpr;
use luaur_ast::records::ast_table_indexer::AstTableIndexer;
use luaur_ast::visit;

impl TypeMapVisitor<'_> {
    pub fn visit_ast_expr_index_expr(&mut self, node: *mut AstExprIndexExpr) -> bool {
        unsafe {
            let expr = (*node).expr;
            let index = (*node).index;

            visit::ast_expr_visit(expr, self);
            visit::ast_expr_visit(index, self);

            if !self.try_get_table_indexer(expr).is_null() {
                let indexer = self.try_get_table_indexer(expr);
                let result_type = (*indexer).result_type;
                self.record_resolved_type_ast_expr_ast_type(
                    node as *mut luaur_ast::records::ast_expr::AstExpr,
                    result_type,
                );
            }
        }

        false
    }
}
