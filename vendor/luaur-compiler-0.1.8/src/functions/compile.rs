use alloc::string::String;
use luaur_ast::records::allocator::Allocator;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_ast::records::parse_options::ParseOptions;
use luaur_ast::records::parse_result::ParseResult;
use luaur_ast::records::parser::Parser;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;
use luaur_bytecode::records::bytecode_encoder::BytecodeEncoder;
use luaur_common::functions::format::format;
use luaur_common::macros::luau_timetrace_scope::LUAU_TIMETRACE_SCOPE;

use crate::functions::compile_or_throw_compiler::compile_or_throw_bytecode_builder_parse_result_ast_name_table_compile_options;
use crate::records::compile_error::CompileError;
use crate::records::compile_options::CompileOptions;

pub fn compile(
    source: &String,
    options: &CompileOptions,
    parse_options: &ParseOptions,
    encoder: *mut dyn BytecodeEncoder,
) -> String {
    LUAU_TIMETRACE_SCOPE!("compile", "Compiler");

    let mut allocator = Allocator::allocator();
    let mut names = AstNameTable::new(&mut allocator);
    let result: ParseResult = Parser::parse(
        source.as_str(),
        source.len(),
        &mut names,
        &mut allocator,
        parse_options.clone(),
    );

    if !result.errors.is_empty() {
        let parse_error = &result.errors[0];
        let error = format!(
            ":{}: {}",
            parse_error.get_location().begin.line + 1,
            parse_error.what()
        );
        return BytecodeBuilder::get_error(&error);
    }

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut bcb = BytecodeBuilder::new(Some(encoder));
        compile_or_throw_bytecode_builder_parse_result_ast_name_table_compile_options(
            &mut bcb, &result, &mut names, options,
        );
        bcb.get_bytecode().clone()
    })) {
        Ok(bytecode) => bytecode,
        Err(payload) => {
            // C++ `catch (CompileError& e)`: the panic payload IS the thrown
            // CompileError; recover its location/message rather than raising anew.
            let compile_error = payload
                .downcast::<CompileError>()
                .expect("compile() caught a non-CompileError panic");
            let compile_error_location = compile_error.get_location();
            let error = format!(
                ":{}: {}",
                compile_error_location.begin.line + 1,
                unsafe { core::ffi::CStr::from_ptr(compile_error.what()) }.to_string_lossy()
            );
            BytecodeBuilder::get_error(&error)
        }
    }
}
