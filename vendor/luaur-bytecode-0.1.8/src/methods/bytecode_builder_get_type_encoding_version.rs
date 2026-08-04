use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_bytecode_tag::LBC_TYPE_VERSION_TARGET;

impl BytecodeBuilder {
    pub fn get_type_encoding_version(&self) -> u8 {
        // C++: `return LBC_TYPE_VERSION_TARGET;` (3). Was stubbed to 1, which made
        // the Rust compiler emit a v1 typeinfo header the VM loader can't parse.
        LBC_TYPE_VERSION_TARGET.0 as u8
    }
}
