use crate::records::bytecode_builder::BytecodeBuilder;

#[test]
fn vector_double_bytecode_version_is_dark_by_default() {
    let old = luaur_common::FFlag::LuauCompileEmitVectorDouble.get();
    let old_classes = luaur_common::FFlag::DebugLuauUserDefinedClasses.get();
    luaur_common::FFlag::DebugLuauUserDefinedClasses.set(false);

    luaur_common::FFlag::LuauCompileEmitVectorDouble.set(false);
    assert_eq!(BytecodeBuilder::new(None).get_version(), 9);

    luaur_common::FFlag::LuauCompileEmitVectorDouble.set(true);
    assert_eq!(BytecodeBuilder::new(None).get_version(), 13);

    luaur_common::FFlag::LuauCompileEmitVectorDouble.set(old);
    luaur_common::FFlag::DebugLuauUserDefinedClasses.set(old_classes);
}
