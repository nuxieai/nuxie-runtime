use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use luaur_ast::records::ast_array::AstArray;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_stat_return::AstStatReturn;

impl TableMutationTrackerDeprecated<'_> {
    pub fn visit_ast_stat_return(&mut self, node: *mut AstStatReturn) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let node_ref = &*node;
            let list = node_ref.list;

            for i in 0..list.size {
                let expr = unsafe { *list.data.add(i) };
                self.observe_mutations(expr, self.could_be_table_reference(expr));
            }
        }

        false
    }
}
