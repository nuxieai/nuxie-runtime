use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn get_import_id(id0: i32) -> u32 {
        luaur_common::LUAU_ASSERT!((id0 as u32) < 1024);
        (1u32 << 30) | ((id0 as u32) << 20)
    }

    pub fn get_import_id2(id0: i32, id1: i32) -> u32 {
        luaur_common::LUAU_ASSERT!(((id0 | id1) as u32) < 1024);
        (2u32 << 30) | ((id0 as u32) << 20) | ((id1 as u32) << 10)
    }

    pub fn get_import_id3(id0: i32, id1: i32, id2: i32) -> u32 {
        luaur_common::LUAU_ASSERT!(((id0 | id1 | id2) as u32) < 1024);
        (3u32 << 30) | ((id0 as u32) << 20) | ((id1 as u32) << 10) | id2 as u32
    }
}
