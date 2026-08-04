use crate::enums::global::Global;
use crate::functions::analyze_builtins::analyze_builtins;
use crate::functions::assign_mutable::assign_mutable;
use crate::functions::build_table_constant_map::build_table_constant_map;
use crate::functions::build_type_map::build_type_map;
use crate::functions::fold_constants::fold_constants;
use crate::functions::get_global_state::get_global_state;
use crate::functions::predict_table_shapes::predict_table_shapes;
use crate::functions::set_compile_options_for_native_compilation::set_compile_options_for_native_compilation;
use crate::functions::track_values::track_values;
use crate::records::compile_error::CompileError;
use crate::records::compile_options::CompileOptions;
use crate::records::compiler::Compiler;
use crate::records::fenv_visitor::FenvVisitor;
use crate::records::function_visitor::FunctionVisitor;
use luaur_ast::records::ast_array::AstArray;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_stat::AstStat;
use luaur_ast::records::parse_result::ParseResult;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_bytecode_type::{
    LBC_TYPE_TAGGED_USERDATA_BASE, LBC_TYPE_TAGGED_USERDATA_END,
};
use luaur_common::enums::luau_proto_flag::LuauProtoFlag;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_timetrace_scope::LUAU_TIMETRACE_SCOPE;

pub fn compile_or_throw_bytecode_builder_parse_result_ast_name_table_compile_options(
    bytecode: &mut BytecodeBuilder,
    parse_result: &ParseResult,
    names: &mut AstNameTable,
    input_options: &CompileOptions,
) {
    LUAU_TIMETRACE_SCOPE!("compileOrThrow", "Compiler");

    LUAU_ASSERT!(!parse_result.root.is_null());
    LUAU_ASSERT!(parse_result.errors.is_empty());

    let mut options = *input_options;
    let mut main_flags = 0u8;

    for hc in &parse_result.hotcomments {
        if hc.header {
            if let Some(value) = hc.content.strip_prefix("optimize ") {
                let level = value.parse::<i32>().unwrap_or(0).clamp(0, 2);
                options.optimization_level = level;
            }

            if hc.content == "native" {
                main_flags |= LuauProtoFlag::LPF_NATIVE_MODULE as u8;
                set_compile_options_for_native_compilation(&mut options);
            }
        }
    }

    let root = parse_result.root;
    let root_node = root as *mut AstNode;

    let mut functions = Vec::<*mut AstExprFunction>::new();
    {
        let mut function_visitor = FunctionVisitor::new(&mut functions);
        unsafe {
            luaur_ast::visit::ast_stat_visit(root as *mut AstStat, &mut function_visitor);
        }

        if function_visitor.has_native_function {
            set_compile_options_for_native_compilation(&mut options);
        }
    }

    let mut compiler = Compiler::compiler(bytecode, &options, names);

    assign_mutable(&mut compiler.globals, names, options.mutable_globals);
    track_values(&mut compiler.globals, &mut compiler.variables, root_node);

    if options.optimization_level >= 1
        && (!names.get(c"getfenv".as_ptr()).value.is_null()
            || !names.get(c"setfenv".as_ptr()).value.is_null())
    {
        let mut fenv_visitor =
            FenvVisitor::fenv_visitor(&mut compiler.getfenv_used, &mut compiler.setfenv_used);
        unsafe {
            luaur_ast::visit::ast_stat_visit(root as *mut AstStat, &mut fenv_visitor);
        }
    }

    if options.optimization_level >= 2 && !compiler.getfenv_used && !compiler.setfenv_used {
        compiler.builtins_fold = &compiler.builtins as *const _;

        let math = names.get(c"math".as_ptr());
        if !math.value.is_null() && get_global_state(&compiler.globals, math) == Global::Default {
            compiler.builtins_fold_library_k = true;
        } else if !options.libraries_with_known_members.is_null() {
            unsafe {
                let mut ptr = options.libraries_with_known_members;
                while !(*ptr).is_null() {
                    let name = names.get(*ptr);
                    if !name.value.is_null()
                        && get_global_state(&compiler.globals, name) == Global::Default
                    {
                        compiler.builtins_fold_library_k = true;
                        break;
                    }
                    ptr = ptr.add(1);
                }
            }
        }
    }

    if options.optimization_level >= 1 {
        analyze_builtins(
            &mut compiler.builtins,
            &compiler.globals,
            &compiler.variables,
            &options,
            root_node,
            names,
        );

        if luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
            && luaur_common::FFlag::LuauCompileFoldOptimize.get()
        {
            build_table_constant_map(
                &mut compiler.table_constants,
                &compiler.variables,
                root_node,
            );
        }

        fold_constants(
            &mut compiler.constants,
            &mut compiler.variables,
            &mut compiler.locstants,
            compiler.builtins_fold,
            compiler.builtins_fold_library_k,
            options.library_member_constant_cb,
            root_node,
            names,
            &compiler.table_constants,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        predict_table_shapes(&mut compiler.table_shapes, root_node);
    }

    if !options.userdata_types.is_null() {
        unsafe {
            let mut ptr = options.userdata_types;
            while !(*ptr).is_null() {
                let name = names.get(*ptr);
                if !name.value.is_null() {
                    let name_str = core::ffi::CStr::from_ptr(name.value).to_string_lossy();
                    *compiler.userdata_types.get_or_insert(name) =
                        bytecode.add_userdata_type(&name_str) as u8;
                }
                ptr = ptr.add(1);
            }

            let count = ptr.offset_from(options.userdata_types) as u16;
            if count > (LBC_TYPE_TAGGED_USERDATA_END.0 - LBC_TYPE_TAGGED_USERDATA_BASE.0) {
                CompileError::raise(
                    &(*root).base.base.location,
                    format_args!("Exceeded userdata type limit in the compilation options"),
                );
            }
        }
    }

    if options.type_info_level >= 1 || options.optimization_level >= 2 {
        build_type_map(
            &mut compiler.function_types,
            &mut compiler.local_types,
            &mut compiler.expr_types,
            root_node,
            options.vector_type,
            &compiler.userdata_types,
            &compiler.builtin_types,
            &compiler.builtins,
            &compiler.globals,
            options.library_member_type_cb,
            bytecode,
        );
    }

    for expr in functions {
        let mut protoflags = 0u8;
        compiler.compile_function(expr, &mut protoflags);

        if (protoflags & LuauProtoFlag::LPF_NATIVE_FUNCTION as u8) != 0
            && (main_flags & LuauProtoFlag::LPF_NATIVE_MODULE as u8) == 0
        {
            main_flags |= LuauProtoFlag::LPF_NATIVE_FUNCTION as u8;
        }
    }

    let mut main = unsafe {
        AstExprFunction::new(
            (*root).base.base.location,
            AstArray::default(),
            AstArray::default(),
            AstArray::default(),
            core::ptr::null_mut(),
            AstArray::default(),
            true,
            Default::default(),
            root,
            0,
            AstName::default(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            None,
        )
    };

    let mainid = compiler.compile_function(&mut main, &mut main_flags);

    let main_ptr = &mut main as *mut AstExprFunction;
    let mainf = compiler.functions.find(&main_ptr);
    LUAU_ASSERT!(mainf.map_or(false, |f| f.upvals.is_empty()));

    bytecode.set_main_function(mainid);
    bytecode.finalize();
}
