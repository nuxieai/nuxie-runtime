use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_local::AstLocal;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

const K_DEFAULT_ALLOC_PC: u32 = !0u32;
const K_MAX_LOCAL_COUNT: usize = 200;

impl Compiler {
    pub fn push_local(&mut self, local: *mut AstLocal, reg: u8, allocpc: u32) {
        if self.local_stack.len() >= K_MAX_LOCAL_COUNT {
            let local_ref = unsafe { &*local };
            let name = unsafe { core::ffi::CStr::from_ptr(local_ref.name.value) }.to_string_lossy();
            CompileError::raise(
                &local_ref.location,
                format_args!(
                    "Out of local registers when trying to allocate {}: exceeded limit {}",
                    name, K_MAX_LOCAL_COUNT
                ),
            );
        }

        self.local_stack.push(local);

        let debugpc = unsafe { (*self.bytecode).get_debug_pc() };
        let l = self.locals.get_or_insert(local);
        LUAU_ASSERT!(!l.allocated);

        l.reg = reg;
        l.allocated = true;
        l.debugpc = debugpc;
        l.allocpc = if allocpc == K_DEFAULT_ALLOC_PC {
            l.debugpc
        } else {
            allocpc
        };
    }
}
