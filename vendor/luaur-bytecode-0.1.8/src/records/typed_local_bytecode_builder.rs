use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypedLocal {
    pub(crate) r#type: LuauBytecodeType,
    pub(crate) reg: u8,
    pub(crate) startpc: u32,
    pub(crate) endpc: u32,
}
