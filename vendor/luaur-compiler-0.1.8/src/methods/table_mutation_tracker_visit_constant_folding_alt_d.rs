use crate::records::table_mutation_tracker::TableMutationTracker;
use luaur_ast::records::ast_stat_assign::AstStatAssign;
use luaur_ast::records::ast_visitor::AstVisitor;

impl<'a> TableMutationTracker<'a> {
    pub fn visit_ast_stat_assign(&mut self, node: *mut AstStatAssign) -> bool {
        if node.is_null() {
            return false;
        }

        unsafe {
            let node_ref = &*node;

            for rhs in node_ref.values.as_slice() {
                self.mark_escaped_impl(*rhs);
            }

            for lhs in node_ref.vars.as_slice() {
                self.mark_escaped_table_index(*lhs, true);
            }
        }

        true
    }
}
