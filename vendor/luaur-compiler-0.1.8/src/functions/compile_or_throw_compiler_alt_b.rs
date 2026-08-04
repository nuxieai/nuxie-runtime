use crate::functions::compile_or_throw_compiler::compile_or_throw_bytecode_builder_parse_result_ast_name_table_compile_options;
use crate::records::compile_options::CompileOptions;
use alloc::string::String;
use luaur_ast::records::allocator::Allocator;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_ast::records::parse_options::ParseOptions;
use luaur_ast::records::parser::Parser;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;

pub fn compile_or_throw_bytecode_builder_string_compile_options_parse_options(
    bytecode: &mut BytecodeBuilder,
    source: &String,
    options: &CompileOptions,
    parse_options: &ParseOptions,
) {
    let mut allocator = Allocator::allocator();
    let mut names = AstNameTable::new(&mut allocator);
    let result = Parser::parse(
        source.as_str(),
        source.len(),
        &mut names,
        &mut allocator,
        parse_options.clone(),
    );

    if !result.errors.is_empty() {
        // Faithful to C++ `throw ParseErrors(result.errors)`: panic with the
        // ParseErrors payload (not a bare "ParseErrors" string) so callers that
        // catch the unwind can downcast it and read the real message via Display
        // (`ParseErrors::what()` = the first error's message for a single error).
        std::panic::panic_any(luaur_ast::records::parse_errors::ParseErrors::new(
            result.errors.clone(),
        ));
    }

    compile_or_throw_bytecode_builder_parse_result_ast_name_table_compile_options(
        bytecode, &result, &mut names, options,
    );
}
