use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypedUpval {
    pub(crate) r#type: LuauBytecodeType,
}
