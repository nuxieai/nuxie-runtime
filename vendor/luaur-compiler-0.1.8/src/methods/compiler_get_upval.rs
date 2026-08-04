use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_local::AstLocal;

const K_MAX_UPVALUE_COUNT: usize = 200;

impl Compiler {
    pub fn get_upval(&mut self, local: *mut AstLocal) -> u8 {
        for (uid, &upval) in self.upvals.iter().enumerate() {
            if upval == local {
                return uid as u8;
            }
        }

        if self.upvals.len() >= K_MAX_UPVALUE_COUNT {
            let local_ref = unsafe { &*local };
            let name = unsafe { core::ffi::CStr::from_ptr(local_ref.name.value) }.to_string_lossy();
            CompileError::raise(
                &local_ref.location,
                format_args!(
                    "Out of upvalue registers when trying to allocate {}: exceeded limit {}",
                    name, K_MAX_UPVALUE_COUNT
                ),
            );
        }

        if self.variables.find(&local).map_or(false, |v| v.written) {
            self.locals.get_or_insert(local).captured = true;
        }

        self.upvals.push(local);
        (self.upvals.len() - 1) as u8
    }
}
