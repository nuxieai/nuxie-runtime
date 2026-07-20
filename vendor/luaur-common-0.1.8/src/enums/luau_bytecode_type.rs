//! Generated skeleton item.
//! Node: `cxx:Enum:Luau.Common:Common/include/Luau/Bytecode.h:526:luau_bytecode_type`
//! Source: `Common/include/Luau/Bytecode.h`
//! Graph edges:
//! - declared_by: source_file Common/include/Luau/Bytecode.h
//! - incoming:
//!   - declares <- source_file Common/include/Luau/Bytecode.h

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LuauBytecodeType(pub u16);

impl crate::records::dense_hash_table::DenseDefault for LuauBytecodeType {
    fn dense_default() -> Self {
        LBC_TYPE_NIL
    }
}

pub const LBC_TYPE_NIL: LuauBytecodeType = LuauBytecodeType(0);
pub const LBC_TYPE_BOOLEAN: LuauBytecodeType = LuauBytecodeType(1);
pub const LBC_TYPE_NUMBER: LuauBytecodeType = LuauBytecodeType(2);
pub const LBC_TYPE_STRING: LuauBytecodeType = LuauBytecodeType(3);
pub const LBC_TYPE_TABLE: LuauBytecodeType = LuauBytecodeType(4);
pub const LBC_TYPE_FUNCTION: LuauBytecodeType = LuauBytecodeType(5);
pub const LBC_TYPE_THREAD: LuauBytecodeType = LuauBytecodeType(6);
pub const LBC_TYPE_USERDATA: LuauBytecodeType = LuauBytecodeType(7);
pub const LBC_TYPE_VECTOR: LuauBytecodeType = LuauBytecodeType(8);
pub const LBC_TYPE_BUFFER: LuauBytecodeType = LuauBytecodeType(9);
pub const LBC_TYPE_INTEGER: LuauBytecodeType = LuauBytecodeType(10);

pub const LBC_TYPE_ANY: LuauBytecodeType = LuauBytecodeType(15);

pub const LBC_TYPE_TAGGED_USERDATA_BASE: LuauBytecodeType = LuauBytecodeType(64);
pub const LBC_TYPE_TAGGED_USERDATA_END: LuauBytecodeType = LuauBytecodeType(64 + 32);

pub const LBC_TYPE_OPTIONAL_BIT: LuauBytecodeType = LuauBytecodeType(1 << 7);

pub const LBC_TYPE_INVALID: LuauBytecodeType = LuauBytecodeType(256);

// Also exposed as associated consts so `LuauBytecodeType::LBC_TYPE_X` compiles —
// that is how the model naturally scopes a C++ enum constant, and the module-const
// form alone produced a persistent E0599 class no prompt/card note could fix. Each
// associated const names the same module const (one source of truth), so the bare
// `LBC_TYPE_X` that existing code imports still works unchanged.
impl LuauBytecodeType {
    pub const LBC_TYPE_NIL: LuauBytecodeType = LBC_TYPE_NIL;
    pub const LBC_TYPE_BOOLEAN: LuauBytecodeType = LBC_TYPE_BOOLEAN;
    pub const LBC_TYPE_NUMBER: LuauBytecodeType = LBC_TYPE_NUMBER;
    pub const LBC_TYPE_STRING: LuauBytecodeType = LBC_TYPE_STRING;
    pub const LBC_TYPE_TABLE: LuauBytecodeType = LBC_TYPE_TABLE;
    pub const LBC_TYPE_FUNCTION: LuauBytecodeType = LBC_TYPE_FUNCTION;
    pub const LBC_TYPE_THREAD: LuauBytecodeType = LBC_TYPE_THREAD;
    pub const LBC_TYPE_USERDATA: LuauBytecodeType = LBC_TYPE_USERDATA;
    pub const LBC_TYPE_VECTOR: LuauBytecodeType = LBC_TYPE_VECTOR;
    pub const LBC_TYPE_BUFFER: LuauBytecodeType = LBC_TYPE_BUFFER;
    pub const LBC_TYPE_INTEGER: LuauBytecodeType = LBC_TYPE_INTEGER;
    pub const LBC_TYPE_ANY: LuauBytecodeType = LBC_TYPE_ANY;
    pub const LBC_TYPE_TAGGED_USERDATA_BASE: LuauBytecodeType = LBC_TYPE_TAGGED_USERDATA_BASE;
    pub const LBC_TYPE_TAGGED_USERDATA_END: LuauBytecodeType = LBC_TYPE_TAGGED_USERDATA_END;
    pub const LBC_TYPE_OPTIONAL_BIT: LuauBytecodeType = LBC_TYPE_OPTIONAL_BIT;
    pub const LBC_TYPE_INVALID: LuauBytecodeType = LBC_TYPE_INVALID;
}
