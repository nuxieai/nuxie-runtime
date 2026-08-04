use crate::records::table_mutation_tracker::TableMutationTracker;
use luaur_ast::records::ast_stat_return::AstStatReturn;

impl<'a> TableMutationTracker<'a> {
    pub fn visit_ast_stat_return(&mut self, node: *mut AstStatReturn) -> bool {
        if node.is_null() {
            return false;
        }

        unsafe {
            let node_ref = &*node;

            for expr_ptr in node_ref.list.as_slice() {
                self.mark_escaped_impl(*expr_ptr);
            }
        }

        true
    }
}
