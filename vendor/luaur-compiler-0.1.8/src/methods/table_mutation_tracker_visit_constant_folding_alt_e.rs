use crate::records::table_mutation_tracker::TableMutationTracker;
use luaur_ast::records::ast_stat_compound_assign::AstStatCompoundAssign;
use luaur_ast::records::ast_visitor::AstVisitor;

impl<'a> TableMutationTracker<'a> {
    pub fn visit_ast_stat_compound_assign(&mut self, node: *mut AstStatCompoundAssign) -> bool {
        if node.is_null() {
            return true;
        }

        unsafe {
            let node_ref = &*node;

            // LHS index expressions mutate the table
            self.mark_escaped_table_index(node_ref.var, true);
        }

        true
    }
}
