use crate::records::builtin_ast_types::BuiltinAstTypes;
use crate::records::compile_options::CompileOptions;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_ast::records::location::Location;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl Compiler {
    pub fn compiler(
        bytecode: &mut BytecodeBuilder,
        options: &CompileOptions,
        names: &mut AstNameTable,
    ) -> Compiler {
        let export_name = names.get_or_add_c_str(c"__EXP".as_ptr());

        let mut compiler = Compiler {
            bytecode: bytecode as *mut BytecodeBuilder,
            options: *options,
            functions: DenseHashMap::new(core::ptr::null_mut()),
            locals: DenseHashMap::new(core::ptr::null_mut()),
            globals: DenseHashMap::new(AstName::default()),
            variables: DenseHashMap::new(core::ptr::null_mut()),
            constants: DenseHashMap::new(core::ptr::null_mut()),
            locstants: DenseHashMap::new(core::ptr::null_mut()),
            table_constants: DenseHashMap::new(core::ptr::null_mut()),
            table_shapes: DenseHashMap::new(core::ptr::null_mut()),
            builtins: DenseHashMap::new(core::ptr::null_mut()),
            userdata_types: DenseHashMap::new(AstName::default()),
            function_types: DenseHashMap::new(core::ptr::null_mut()),
            local_types: DenseHashMap::new(core::ptr::null_mut()),
            expr_types: DenseHashMap::new(core::ptr::null_mut()),
            inline_builtins: DenseHashMap::new(core::ptr::null_mut()),
            inline_builtins_backup: DenseHashMap::new(core::ptr::null_mut()),
            expr_changes: Vec::new(),
            local_changes: Vec::new(),
            builtin_types: BuiltinAstTypes::new(options.vector_type),
            names: names as *mut AstNameTable,
            export_table_local: AstLocal::new(
                export_name,
                Location::default(),
                core::ptr::null_mut(),
                0,
                0,
                core::ptr::null_mut(),
                true,
            ),
            builtins_fold: core::ptr::null(),
            builtins_fold_library_k: false,
            reg_top: 0,
            stack_size: 0,
            arg_count: 0,
            has_loops: false,
            current_function: core::ptr::null_mut(),
            block_depth: 0,
            getfenv_used: false,
            setfenv_used: false,
            local_stack: Vec::new(),
            upvals: Vec::new(),
            loop_jumps: Vec::new(),
            loops: Vec::new(),
            inline_frames: Vec::new(),
            captures: Vec::new(),
            exported_locals: Vec::new(),
            exported_classes: Vec::new(),
        };

        compiler.local_stack.reserve(16);
        compiler.upvals.reserve(16);
        compiler
    }
}

pub fn compiler_compiler(
    bytecode: &mut BytecodeBuilder,
    options: &CompileOptions,
    names: &mut AstNameTable,
) -> Compiler {
    Compiler::compiler(bytecode, options, names)
}
