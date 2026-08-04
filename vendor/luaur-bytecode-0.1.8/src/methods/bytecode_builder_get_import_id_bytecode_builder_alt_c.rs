//! Node: `cxx:Method:Luau.Bytecode:Bytecode/src/BytecodeBuilder.cpp:1071:getImportId`
//!
//! Three-id overload of `BytecodeBuilder::getImportId` — packs three 10-bit
//! import ids into a 32-bit import constant tagged with count `3` in the top
//! two bits. Faithful to the C++ bit layout; signature-pinned name for the
//! three-arg definition at line 1071 (the overload set is also emitted as
//! `get_import_id3` in the primary file).

use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn get_import_id_i32_i32_i32(id0: i32, id1: i32, id2: i32) -> u32 {
        luaur_common::LUAU_ASSERT!(((id0 | id1 | id2) as u32) < 1024);
        (3u32 << 30) | ((id0 as u32) << 20) | ((id1 as u32) << 10) | id2 as u32
    }
}
