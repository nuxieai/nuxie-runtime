use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use luaur_ast::records::ast_stat_function::AstStatFunction;
use luaur_ast::records::ast_expr::AstExpr;

impl TableMutationTrackerDeprecated<'_> {
    pub fn visit_ast_stat_function(&mut self, node: *mut AstStatFunction) -> bool {
        if node.is_null() {
            return false;
        }

        unsafe {
            let node_ref = &*node;

            // Mutations in the body of the function will get caught by other visitor cases
            self.observe_mutations(node_ref.func as *const AstExpr, false);

            // If this stat adds a table method, the table is no longer constant
            self.observe_mutations(node_ref.name as *const AstExpr, true);
        }

        false
    }
}
