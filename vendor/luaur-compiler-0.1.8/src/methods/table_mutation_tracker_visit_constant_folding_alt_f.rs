use crate::records::table_mutation_tracker::TableMutationTracker;
use luaur_ast::records::ast_stat_function::AstStatFunction;

impl<'a> TableMutationTracker<'a> {
    pub fn visit_ast_stat_function(&mut self, node: *mut AstStatFunction) -> bool {
        if node.is_null() {
            return true;
        }

        unsafe {
            let node_ref = &*node;

            // Adding a method on a table mutates it
            self.mark_escaped_table_index(node_ref.name, true);
        }

        true
    }
}
