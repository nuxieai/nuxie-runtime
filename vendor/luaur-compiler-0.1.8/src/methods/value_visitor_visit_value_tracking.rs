use crate::records::value_visitor::ValueVisitor;
use luaur_ast::records::ast_stat_local::AstStatLocal;

impl ValueVisitor {
    pub fn visit_ast_stat_local(&mut self, node: *mut AstStatLocal) -> bool {
        if node.is_null() {
            return false;
        }

        unsafe {
            let node_ref = &*node;

            let vars_len = node_ref.vars.len();
            let values_len = node_ref.values.len();

            // C++ `variables[vars[i]].init = values[i]`: operator[] creates the entry if
            // absent (default written/constant), but only overwrites `.init` if it exists.
            for i in 0..vars_len.min(values_len) {
                let var_ptr = *node_ref
                    .vars
                    .as_slice()
                    .get(i as usize)
                    .unwrap_or(&core::ptr::null_mut());
                let init_ptr = *node_ref
                    .values
                    .as_slice()
                    .get(i as usize)
                    .unwrap_or(&core::ptr::null_mut());
                self.variables.get_or_insert(var_ptr).init = init_ptr;
            }

            for i in values_len..vars_len {
                let var_ptr = *node_ref
                    .vars
                    .as_slice()
                    .get(i as usize)
                    .unwrap_or(&core::ptr::null_mut());
                self.variables.get_or_insert(var_ptr).init = core::ptr::null_mut();
            }
        }

        true
    }
}
