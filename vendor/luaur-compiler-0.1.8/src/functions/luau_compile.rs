use crate::functions::compile_or_throw_compiler::compile_or_throw_bytecode_builder_parse_result_ast_name_table_compile_options;
use crate::records::compile_options::CompileOptions;
use crate::records::lua_compile_options::LuaCompileOptions;
use core::ffi::c_char;
use luaur_ast::records::allocator::Allocator;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_ast::records::parse_options::ParseOptions;
use luaur_ast::records::parser::Parser;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub fn luau_compile(
    source: *const c_char,
    size: usize,
    options: *mut LuaCompileOptions,
    outsize: *mut usize,
) -> *mut c_char {
    unsafe {
        LUAU_ASSERT!(!outsize.is_null());

        let mut opts = CompileOptions::default();

        if !options.is_null() {
            core::ptr::copy_nonoverlapping(options as *const CompileOptions, &mut opts, 1);
        }

        let source_str = core::slice::from_raw_parts(source as *const u8, size);
        let source_rust = core::str::from_utf8_unchecked(source_str).to_string();

        let mut allocator = Allocator::allocator();
        let mut names = AstNameTable::new(&mut allocator);
        let parse_options = ParseOptions::default();
        let result = Parser::parse(
            source_rust.as_str(),
            source_rust.len(),
            &mut names,
            &mut allocator,
            parse_options,
        );

        let bytecode = if result.errors.is_empty() {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut bcb = BytecodeBuilder::new(None);
                compile_or_throw_bytecode_builder_parse_result_ast_name_table_compile_options(
                    &mut bcb, &result, &mut names, &opts,
                );
                bcb.get_bytecode().clone()
            })) {
                Ok(bc) => bc,
                Err(_) => {
                    let error = ":0: compilation failed";
                    BytecodeBuilder::get_error(error)
                }
            }
        } else {
            let parse_error = &result.errors[0];
            let error = alloc::format!(
                ":{}: {}",
                parse_error.get_location().begin.line + 1,
                parse_error.what()
            );
            BytecodeBuilder::get_error(&error)
        };

        extern "C" {
            fn malloc(size: usize) -> *mut core::ffi::c_void;
        }
        let copy = unsafe { malloc(bytecode.len()) } as *mut c_char;
        if copy.is_null() {
            return core::ptr::null_mut();
        }

        core::ptr::copy_nonoverlapping(bytecode.as_ptr() as *const c_char, copy, bytecode.len());
        *outsize = bytecode.len();
        copy
    }
}
