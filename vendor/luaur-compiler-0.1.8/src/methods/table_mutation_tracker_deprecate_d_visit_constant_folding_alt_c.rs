use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use luaur_ast::records::ast_stat_assign::AstStatAssign;

impl TableMutationTrackerDeprecated<'_> {
    pub fn visit_ast_stat_assign(&mut self, node: *mut AstStatAssign) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let node_ref = &*node;

            let vars_size = node_ref.vars.len();
            let values_size = node_ref.values.len();
            let limit = if vars_size < values_size {
                vars_size
            } else {
                values_size
            };

            let values_slice = node_ref.values.as_slice();

            for i in 0..limit {
                let rhs = values_slice[i];
                self.observe_mutations(
                    rhs,
                    /* could_mutate_table */ self.could_be_table_reference(rhs),
                );
            }

            // Any remaining values don't inherently mutate tables, but we still observe for things like function calls that could mutate tables
            if values_size > vars_size {
                for i in vars_size..values_size {
                    self.observe_mutations(
                        values_slice[i],
                        /* could_mutate_table */ false,
                    );
                }
            }

            // Tables referred to in lhs expressions could be mutated by the assignment
            for lhs in node_ref.vars.as_slice() {
                self.observe_mutations(*lhs, /* could_mutate_table */ true);
            }
        }

        false
    }
}
