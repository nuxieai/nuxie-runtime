use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::FFlag;

impl BytecodeBuilder {
    pub fn get_version(&self) -> u8 {
        if FFlag::DebugLuauUserDefinedClasses.get() {
            return luaur_common::enums::luau_bytecode_tag::LBC_VERSION_CLASSES.0 as u8;
        }

        if FFlag::LuauCompileEmitVectorDouble.get() {
            return 13;
        }

        if FFlag::LuauBytecodeCostModel.get() {
            return 12;
        }

        if FFlag::LuauEmitCallFeedback.get() {
            return 11;
        }

        luaur_common::enums::luau_bytecode_tag::LBC_VERSION_TARGET.0 as u8
    }
}
