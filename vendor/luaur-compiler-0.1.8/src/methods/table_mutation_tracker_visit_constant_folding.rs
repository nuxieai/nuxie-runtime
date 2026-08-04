use crate::records::table_mutation_tracker::TableMutationTracker;
use luaur_ast::records::ast_array::AstArray;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;

impl<'a> TableMutationTracker<'a> {
    pub fn visit_ast_expr_call(&mut self, node: *mut AstExprCall) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let call_node = &mut *node;

            // Values passed in as arguments can escape
            for arg_ptr in call_node.args.as_slice() {
                self.mark_escaped_impl(*arg_ptr);
            }

            // Table indexed in a self call escapes through 'self'
            if call_node.self_ {
                self.mark_escaped_table_index(call_node.func, false);
            }
        }

        true
    }
}
