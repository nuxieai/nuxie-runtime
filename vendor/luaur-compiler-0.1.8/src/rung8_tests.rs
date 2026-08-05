use crate::functions::compile_or_throw_compiler_alt_b::compile_or_throw_bytecode_builder_string_compile_options_parse_options;
use crate::records::compile_options::CompileOptions;
use luaur_ast::records::parse_options::ParseOptions;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;

static FLAG_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn class_inheritance_emits_newclass() {
    let _guard = FLAG_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
    let old = luaur_common::FFlag::DebugLuauUserDefinedClasses.get();
    luaur_common::FFlag::DebugLuauUserDefinedClasses.set(true);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let source = String::from("class Parent\nend\nclass Child extends Parent\nend\nreturn Child");
        let mut bytecode = BytecodeBuilder::new(None);
        bytecode.set_dump_flags(BytecodeBuilder::DUMP_CODE);
        compile_or_throw_bytecode_builder_string_compile_options_parse_options(
            &mut bytecode,
            &source,
            &CompileOptions::default(),
            &ParseOptions::default(),
        );

        (bytecode.get_bytecode().clone(), bytecode.dump_everything())
    }));

    luaur_common::FFlag::DebugLuauUserDefinedClasses.set(old);
    let (bytecode, dump) = result.expect("class inheritance compilation should succeed");
    assert!(!bytecode.starts_with('\0'));
    assert!(dump.contains("NEWCLASS R0 R255"));
    assert!(dump.contains("NEWCLASS R1 R0"));
}

#[test]
fn export_table_optimization_is_dark_by_default() {
    let _guard = FLAG_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
    let old_syntax = luaur_common::FFlag::LuauExportValueSyntax.get();
    let old_optimize = luaur_common::FFlag::LuauOptimizeExportTable.get();
    luaur_common::FFlag::LuauExportValueSyntax.set(true);

    let compile = |optimize: bool| {
        luaur_common::FFlag::LuauOptimizeExportTable.set(optimize);
        let mut bytecode = BytecodeBuilder::new(None);
        bytecode.set_dump_flags(BytecodeBuilder::DUMP_CODE);
        compile_or_throw_bytecode_builder_string_compile_options_parse_options(
            &mut bytecode,
            &String::from("export local x = 5"),
            &CompileOptions::default(),
            &ParseOptions::default(),
        );
        bytecode.dump_everything()
    };

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (compile(false), compile(true))
    }));

    luaur_common::FFlag::LuauExportValueSyntax.set(old_syntax);
    luaur_common::FFlag::LuauOptimizeExportTable.set(old_optimize);
    let (legacy, optimized) = result.expect("both export paths should compile");
    assert!(legacy.contains("NEWTABLE"));
    assert!(optimized.contains("DUPTABLE"));
}
