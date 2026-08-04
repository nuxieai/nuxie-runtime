use crate::records::cost::Cost;
use crate::records::cost_visitor::CostVisitor;
use luaur_ast::records::ast_stat_assign::AstStatAssign;

impl CostVisitor {
    pub fn visit_ast_stat_assign(&mut self, node: *mut core::ffi::c_void) -> bool {
        unsafe {
            let node = &*(node as *mut AstStatAssign);

            for i in 0..node.vars.len() {
                let var_ptr = *node
                    .vars
                    .as_slice()
                    .get(i as usize)
                    .unwrap_or(&core::ptr::null_mut());
                self.assign(var_ptr);
            }

            let mut i = 0;
            while i < node.vars.len() as usize || i < node.values.len() as usize {
                let mut ac = Cost::default();
                if i < node.vars.len() as usize {
                    let var_ptr = *node
                        .vars
                        .as_slice()
                        .get(i)
                        .unwrap_or(&core::ptr::null_mut());
                    ac = ac.add(&self.model(var_ptr));
                }
                if i < node.values.len() as usize {
                    let val_ptr = *node
                        .values
                        .as_slice()
                        .get(i)
                        .unwrap_or(&core::ptr::null_mut());
                    ac = ac.add(&self.model(val_ptr));
                }

                // local->local or constant->local assignment is not free
                if ac.model == 0 {
                    self.result.operator_add_assign(&Cost::new(1, 0));
                } else {
                    self.result.operator_add_assign(&ac);
                }

                i += 1;
            }
        }

        false
    }
}
