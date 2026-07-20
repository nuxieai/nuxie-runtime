//! Generated skeleton item.
//! Node: `cxx:Enum:Luau.Common:Common/include/Luau/Bytecode.h:498:luau_bytecode_tag`
//! Source: `Common/include/Luau/Bytecode.h`
//! Graph edges:
//! - declared_by: source_file Common/include/Luau/Bytecode.h
//! - incoming:
//!   - declares <- source_file Common/include/Luau/Bytecode.h

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LuauBytecodeTag(pub u32);

impl LuauBytecodeTag {
    /// Bytecode version; runtime supports [MIN, MAX], compiler emits TARGET by default but may emit a higher version when flags are enabled
    pub const LBC_VERSION_MIN: Self = Self(3);
    pub const LBC_VERSION_MAX: Self = Self(11);
    pub const LBC_VERSION_TARGET: Self = Self(6);

    /// Type encoding version
    pub const LBC_TYPE_VERSION_MIN: Self = Self(1);
    pub const LBC_TYPE_VERSION_MAX: Self = Self(3);
    pub const LBC_TYPE_VERSION_TARGET: Self = Self(3);

    /// Types of constant table entries
    pub const LBC_CONSTANT_NIL: Self = Self(0);
    pub const LBC_CONSTANT_BOOLEAN: Self = Self(1);
    pub const LBC_CONSTANT_NUMBER: Self = Self(2);
    pub const LBC_CONSTANT_STRING: Self = Self(3);
    pub const LBC_CONSTANT_IMPORT: Self = Self(4);
    pub const LBC_CONSTANT_TABLE: Self = Self(5);
    pub const LBC_CONSTANT_CLOSURE: Self = Self(6);
    pub const LBC_CONSTANT_VECTOR: Self = Self(7);
    pub const LBC_CONSTANT_TABLE_WITH_CONSTANTS: Self = Self(8);
    pub const LBC_CONSTANT_INTEGER: Self = Self(9);
    pub const LBC_CONSTANT_CLASS_SHAPE: Self = Self(10);

    /** WARNING: This must always be last. */
    pub const LBC_CONSTANT__COUNT: Self = Self(11);
}

pub const LBC_VERSION_MIN: LuauBytecodeTag = LuauBytecodeTag::LBC_VERSION_MIN;
pub const LBC_VERSION_MAX: LuauBytecodeTag = LuauBytecodeTag::LBC_VERSION_MAX;
pub const LBC_VERSION_TARGET: LuauBytecodeTag = LuauBytecodeTag::LBC_VERSION_TARGET;

pub const LBC_TYPE_VERSION_MIN: LuauBytecodeTag = LuauBytecodeTag::LBC_TYPE_VERSION_MIN;
pub const LBC_TYPE_VERSION_MAX: LuauBytecodeTag = LuauBytecodeTag::LBC_TYPE_VERSION_MAX;
pub const LBC_TYPE_VERSION_TARGET: LuauBytecodeTag = LuauBytecodeTag::LBC_TYPE_VERSION_TARGET;

pub const LBC_CONSTANT_NIL: LuauBytecodeTag = LuauBytecodeTag::LBC_CONSTANT_NIL;
pub const LBC_CONSTANT_BOOLEAN: LuauBytecodeTag = LuauBytecodeTag::LBC_CONSTANT_BOOLEAN;
pub const LBC_CONSTANT_NUMBER: LuauBytecodeTag = LuauBytecodeTag::LBC_CONSTANT_NUMBER;
pub const LBC_CONSTANT_STRING: LuauBytecodeTag = LuauBytecodeTag::LBC_CONSTANT_STRING;
pub const LBC_CONSTANT_IMPORT: LuauBytecodeTag = LuauBytecodeTag::LBC_CONSTANT_IMPORT;
pub const LBC_CONSTANT_TABLE: LuauBytecodeTag = LuauBytecodeTag::LBC_CONSTANT_TABLE;
pub const LBC_CONSTANT_CLOSURE: LuauBytecodeTag = LuauBytecodeTag::LBC_CONSTANT_CLOSURE;
pub const LBC_CONSTANT_VECTOR: LuauBytecodeTag = LuauBytecodeTag::LBC_CONSTANT_VECTOR;
pub const LBC_CONSTANT_TABLE_WITH_CONSTANTS: LuauBytecodeTag =
    LuauBytecodeTag::LBC_CONSTANT_TABLE_WITH_CONSTANTS;
pub const LBC_CONSTANT_INTEGER: LuauBytecodeTag = LuauBytecodeTag::LBC_CONSTANT_INTEGER;
pub const LBC_CONSTANT_CLASS_SHAPE: LuauBytecodeTag = LuauBytecodeTag::LBC_CONSTANT_CLASS_SHAPE;

pub const LBC_CONSTANT__COUNT: LuauBytecodeTag = LuauBytecodeTag::LBC_CONSTANT__COUNT;
