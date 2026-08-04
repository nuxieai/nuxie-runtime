use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use luaur_ast::records::ast_stat_for_in::AstStatForIn;
use luaur_ast::records::ast_visitor::AstVisitor;

impl TableMutationTrackerDeprecated<'_> {
    pub fn visit_ast_stat_for_in(&mut self, node: *mut AstStatForIn) -> bool {
        if node.is_null() {
            return false;
        }

        unsafe {
            let node_ref = &*node;

            // Table iterators could mutate their tables
            for expr_ptr in node_ref.values.as_slice() {
                self.observe_mutations(*expr_ptr, true);
            }

            luaur_ast::visit::ast_stat_block_visit(&*node_ref.body, self);
        }

        false
    }
}
