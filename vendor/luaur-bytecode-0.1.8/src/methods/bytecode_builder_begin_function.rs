use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn begin_function(&mut self, numparams: u8, isvararg: bool) -> u32 {
        LUAU_ASSERT!(self.current_function == u32::MAX);

        let id = self.functions.len() as u32;

        let mut func = crate::records::function::Function::default();
        func.numparams = numparams;
        func.isvararg = isvararg;

        self.functions.push(func);

        self.current_function = id;

        self.has_long_jumps = false;
        self.debug_line = 0;

        id
    }
}
