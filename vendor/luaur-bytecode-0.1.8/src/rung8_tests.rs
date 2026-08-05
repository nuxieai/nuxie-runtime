use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::class_shape::ClassShape;
use luaur_common::enums::luau_opcode::LuauOpcode;
use std::sync::Mutex;

static FFLAG_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn vector_double_bytecode_version_is_dark_by_default() {
    let _guard = FFLAG_LOCK.lock().unwrap();
    let old = luaur_common::FFlag::LuauCompileEmitVectorDouble.get();
    let old_classes = luaur_common::FFlag::DebugLuauUserDefinedClasses.get();
    luaur_common::FFlag::DebugLuauUserDefinedClasses.set(false);

    luaur_common::FFlag::LuauCompileEmitVectorDouble.set(false);
    assert_eq!(BytecodeBuilder::new(None).get_version(), 7);

    luaur_common::FFlag::LuauCompileEmitVectorDouble.set(true);
    assert_eq!(BytecodeBuilder::new(None).get_version(), 13);

    luaur_common::FFlag::LuauCompileEmitVectorDouble.set(old);
    luaur_common::FFlag::DebugLuauUserDefinedClasses.set(old_classes);
}

#[test]
fn class_shape_graph_decode_preserves_cpp_resize_then_append_layout() {
    let _guard = FFLAG_LOCK.lock().unwrap();
    let old_classes = luaur_common::FFlag::DebugLuauUserDefinedClasses.get();
    luaur_common::FFlag::DebugLuauUserDefinedClasses.set(true);

    let mut builder = BytecodeBuilder::new(None);
    let function_id = builder.begin_function(0, false);
    assert_eq!(
        builder.add_class_shape(ClassShape {
            className: 7,
            propertyNames: vec![11, 12],
            methodNames: vec![13],
        }),
        0
    );
    builder.emit_abc(LuauOpcode::LOP_RETURN, 0, 1, 0);
    builder.end_function(1, 0, 0, 0);

    let mut strings = Vec::new();
    let mut decoded = crate::functions::from_function_bytecode::from_function_bytecode(
        builder.get_function_data(function_id),
        &mut strings,
    )
    .unwrap();
    assert_eq!(decoded.class_shapes[0].propertyNames, vec![0, 0, 11, 12]);
    assert_eq!(decoded.class_shapes[0].methodNames, vec![0, 13]);

    let roundtripped = crate::functions::to_function_bytecode_bytecode_graph_alt_b::to_function_bytecode_comp_time_bc_function(
        &mut decoded,
    );
    let decoded_again = crate::functions::from_function_bytecode::from_function_bytecode(
        roundtripped,
        &mut strings,
    )
    .unwrap();
    assert_eq!(
        decoded_again.class_shapes[0].propertyNames,
        vec![0, 0, 0, 0, 0, 0, 11, 12]
    );
    assert_eq!(decoded_again.class_shapes[0].methodNames, vec![0, 0, 0, 13]);

    luaur_common::FFlag::DebugLuauUserDefinedClasses.set(old_classes);
}
