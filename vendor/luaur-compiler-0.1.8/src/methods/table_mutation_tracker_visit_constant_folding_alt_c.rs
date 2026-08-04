use crate::records::table_mutation_tracker::TableMutationTracker;
use luaur_ast::records::ast_stat_local::AstStatLocal;
use luaur_ast::records::ast_visitor::AstVisitor;

impl<'a> TableMutationTracker<'a> {
    pub fn visit_ast_stat_local(&mut self, node: *mut AstStatLocal) -> bool {
        if node.is_null() {
            return false;
        }

        unsafe {
            let node_ref = &*node;

            for i in 0..node_ref.values.len().min(node_ref.vars.len()) {
                let value_ptr = *node_ref
                    .values
                    .as_slice()
                    .get(i)
                    .unwrap_or(&core::ptr::null_mut());
                self.mark_escaped_impl(value_ptr);
            }
        }

        true
    }
}
