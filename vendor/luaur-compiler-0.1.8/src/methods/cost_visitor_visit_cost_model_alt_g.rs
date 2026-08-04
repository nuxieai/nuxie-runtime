use crate::records::cost::Cost;
use crate::records::cost_visitor::CostVisitor;
use luaur_ast::records::ast_stat_local::AstStatLocal;

impl CostVisitor {
    pub fn visit_ast_stat_local(&mut self, node: *mut core::ffi::c_void) -> bool {
        unsafe {
            let node = &*(node as *mut AstStatLocal);

            for i in 0..node.values.len() {
                let expr_ptr = *node
                    .values
                    .as_slice()
                    .get(i)
                    .unwrap_or(&core::ptr::null_mut());
                let arg = self.model(expr_ptr);

                if arg.constant != 0 && i < node.vars.len() {
                    let var_ptr = *node
                        .vars
                        .as_slice()
                        .get(i)
                        .unwrap_or(&core::ptr::null_mut());
                    self.vars.try_insert(var_ptr, arg.constant);
                }

                self.result.operator_add_assign(&arg);
            }
        }

        false
    }
}
