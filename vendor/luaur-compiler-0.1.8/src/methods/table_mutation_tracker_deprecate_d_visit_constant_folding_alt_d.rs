use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use luaur_ast::records::ast_stat_compound_assign::AstStatCompoundAssign;

impl TableMutationTrackerDeprecated<'_> {
    pub fn visit_ast_stat_compound_assign(&mut self, node: *mut AstStatCompoundAssign) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let node_ref = &*node;
            let rhs = node_ref.value;

            self.observe_mutations(rhs, self.could_be_table_reference(rhs));
            self.observe_mutations(node_ref.var, true);
        }

        false
    }
}
