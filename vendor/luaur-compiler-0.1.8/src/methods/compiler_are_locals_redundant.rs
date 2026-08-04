use crate::records::compiler::Compiler;
use crate::records::variable::Variable;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_stat_local::AstStatLocal;

impl Compiler {
    pub fn are_locals_redundant(&mut self, stat: *mut AstStatLocal) -> bool {
        unsafe {
            if stat.is_null() {
                return false;
            }

            let stat_ref = &*stat;

            // Extra expressions may have side effects
            if stat_ref.values.len() > stat_ref.vars.len() {
                return false;
            }

            for i in 0..stat_ref.vars.len() {
                let local_ptr = *stat_ref
                    .vars
                    .as_slice()
                    .get(i)
                    .unwrap_or(&core::ptr::null_mut());
                if local_ptr.is_null() {
                    return false;
                }

                let local = &*local_ptr;

                if local.is_exported {
                    // exported locals must be written to the export table
                    return false;
                }

                let v_opt = self.variables.find(&local_ptr);

                let v_ptr = match v_opt {
                    Some(ptr) => ptr,
                    None => return false,
                };

                let v: &Variable = &*v_ptr;
                if !v.constant {
                    return false;
                }
            }
        }

        true
    }
}
