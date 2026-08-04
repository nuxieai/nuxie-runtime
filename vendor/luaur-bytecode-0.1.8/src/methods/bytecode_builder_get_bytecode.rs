use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn get_bytecode(&self) -> &alloc::string::String {
        LUAU_ASSERT!(!self.bytecode.is_empty()); // did you forget to call finalize?
        &self.bytecode
    }
}
