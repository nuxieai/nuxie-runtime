use crate::functions::sref_compiler::sref_ast_name;
use crate::records::compiler::Compiler;
use luaur_common::enums::luau_bytecode_type::LBC_TYPE_ANY;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn pop_locals(&mut self, start: usize) {
        LUAU_ASSERT!(start <= self.local_stack.len());

        for i in start..self.local_stack.len() {
            let local = self.local_stack[i];
            let l = self.locals.find_mut(&local);
            LUAU_ASSERT!(l.is_some());
            let l = l.unwrap();
            LUAU_ASSERT!(l.allocated);

            l.allocated = false;

            if self.options.debug_level >= 2 {
                let debugpc = unsafe { (*self.bytecode).get_debug_pc() };
                unsafe {
                    (*self.bytecode).push_debug_local(
                        sref_ast_name((*local).name),
                        l.reg,
                        l.debugpc,
                        debugpc,
                    );
                }
            }

            if self.options.type_info_level >= 1 && i >= self.arg_count {
                let debugpc = unsafe { (*self.bytecode).get_debug_pc() };
                let ty = self
                    .local_types
                    .find(&local)
                    .copied()
                    .unwrap_or(LBC_TYPE_ANY);

                unsafe {
                    (*self.bytecode).push_local_type_info(ty, l.reg, l.allocpc, debugpc);
                }
            }
        }

        self.local_stack.resize(start, core::ptr::null_mut());
    }
}
