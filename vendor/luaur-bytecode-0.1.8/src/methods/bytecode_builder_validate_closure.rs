use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn validate_closure(&self, cid: i32) -> u8 {
        let proto = unsafe { self.constants[cid as usize].value.valueClosure };
        LUAU_ASSERT!(proto < self.functions.len() as u32);
        self.functions[proto as usize].numupvalues
    }
}
