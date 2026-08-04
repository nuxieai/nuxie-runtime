use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use luaur_ast::records::ast_expr::AstExpr;

impl TableMutationTrackerDeprecated<'_> {
    pub fn visit_ast_expr(&mut self, node: *mut AstExpr) -> bool {
        self.observe_mutations(node as *const AstExpr, false);
        false
    }
}
