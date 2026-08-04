use crate::enums::r#type::Type;
use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn validate_const_type(&self, cid: i32, const_type: Type) {
        LUAU_ASSERT!(
            (cid as usize) < self.constants.len()
                && self.constants[cid as usize].r#type == const_type
        );
    }
}
