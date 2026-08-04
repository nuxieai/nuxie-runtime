use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::location::Location;
use luaur_common::functions::vformat::vformat;
use luaur_common::macros::luau_noinline::LUAU_NOINLINE;

impl Compiler {
    pub fn alloc_reg(&mut self, node: *mut AstNode, count: u32) -> u8 {
        let top = self.reg_top;
        let k_max_register_count = 255;

        if top + count > k_max_register_count {
            let location = unsafe { (*node).location };
            // C++ `CompileError::raise(...)`: throw a typed CompileError (panic_any),
            // not a String panic, so `compile()`'s catch can recover it.
            CompileError::raise(
                &location,
                format_args!(
                    "Out of registers when trying to allocate {} registers: exceeded limit {}",
                    count, k_max_register_count
                ),
            );
        }

        self.reg_top += count;
        self.stack_size = core::cmp::max(self.stack_size, self.reg_top);

        top as u8
    }
}
