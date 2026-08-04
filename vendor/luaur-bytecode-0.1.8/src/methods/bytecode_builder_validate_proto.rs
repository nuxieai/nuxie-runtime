use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn validate_proto(&self, pid: i32) -> u8 {
        LUAU_ASSERT!((pid as usize) < self.protos.len());
        LUAU_ASSERT!(self.protos[pid as usize] < self.functions.len() as u32);
        self.functions[self.protos[pid as usize] as usize].numupvalues
    }
}
