use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn use_userdata_type(&mut self, index: u32) {
        LUAU_ASSERT!((index as usize) < self.userdata_types.len());
        self.userdata_types[index as usize].used = true;
    }
}
