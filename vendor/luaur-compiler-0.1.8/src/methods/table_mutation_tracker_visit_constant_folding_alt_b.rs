use crate::records::table_mutation_tracker::TableMutationTracker;
use luaur_ast::records::ast_expr_table::AstExprTable;

impl<'a> TableMutationTracker<'a> {
    pub fn visit_ast_expr_table(&mut self, node: *mut AstExprTable) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let table_node = &*node;

            for item in table_node.items.as_slice() {
                if !item.key.is_null() {
                    self.mark_escaped_impl(item.key);
                }

                self.mark_escaped_impl(item.value);
            }
        }

        true
    }
}
