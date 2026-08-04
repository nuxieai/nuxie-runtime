use crate::enums::table_constant_kind::TableConstantKind;
use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use luaur_ast::records::ast_stat_local::AstStatLocal;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl TableMutationTrackerDeprecated<'_> {
    pub fn visit_ast_stat_local(&mut self, node: *mut AstStatLocal) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let node_ref = &*node;

            // all values that align wrt indexing are simple - we just match them 1-1
            let vars_size = node_ref.vars.len();
            let values_size = node_ref.values.len();
            let limit = if vars_size < values_size { vars_size } else { values_size };

            let vars_slice = node_ref.vars.as_slice();
            let values_slice = node_ref.values.as_slice();

            for i in 0..limit {
                let local = vars_slice[i];
                let rhs = values_slice[i];

                // note: we rely on trackValues to have been run before us
                // if the local isn't written to, see if we can mark it as a constant
                let v = self.variables.find(&local);
                LUAU_ASSERT!(v.is_some());
                let v = v.unwrap();

                if !v.written {
                    if self.is_constant_table_literal(rhs as *const luaur_ast::records::ast_expr::AstExpr) {
                        *self.constant_tables.get_or_insert(local) = TableConstantKind::ConstantTable;
                    } else if self.is_non_table_constant(rhs) {
                        *self.constant_tables.get_or_insert(local) = TableConstantKind::ConstantOther;
                    }
                }

                // aliasing a table reference could lead to downstream mutations, so we conservatively treat a referenced table as mutated
                if !self.constant_tables.contains(&local) {
                    self.observe_mutations(
                        rhs as *const luaur_ast::records::ast_expr::AstExpr,
                        self.could_be_table_reference(rhs),
                    );
                }
            }

            // check remaining values to observe mutations
            if vars_size < values_size {
                for i in vars_size..values_size {
                    self.observe_mutations(
                        values_slice[i] as *const luaur_ast::records::ast_expr::AstExpr,
                        false,
                    );
                }
            }
        }

        false
    }
}
