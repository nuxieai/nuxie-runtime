use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn validate_const(&self, cid: i32) {
        LUAU_ASSERT!((cid as usize) < self.constants.len());
    }
}
