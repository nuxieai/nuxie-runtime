use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn get_import_id_i32_i32(id0: i32, id1: i32) -> u32 {
        luaur_common::LUAU_ASSERT!(((id0 | id1) as u32) < 1024);
        (2u32 << 30) | ((id0 as u32) << 20) | ((id1 as u32) << 10)
    }
}
