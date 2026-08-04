use crate::records::table_mutation_tracker::TableMutationTracker;
use luaur_ast::records::ast_stat_for_in::AstStatForIn;
use luaur_ast::records::ast_visitor::AstVisitor;

impl<'a> TableMutationTracker<'a> {
    pub fn visit_ast_stat_for_in(&mut self, node: *mut AstStatForIn) -> bool {
        if node.is_null() {
            return false;
        }

        unsafe {
            let node_ref = &*node;

            for expr_ptr in node_ref.values.as_slice() {
                self.mark_escaped_impl(*expr_ptr);
            }
        }

        true
    }
}
