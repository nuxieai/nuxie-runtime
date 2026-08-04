use crate::enums::global::Global;
use crate::records::builtin_ast_types::BuiltinAstTypes;
use crate::records::type_map_visitor::TypeMapVisitor;
use crate::type_aliases::library_member_type_callback::LibraryMemberTypeCallback;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_char;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl<'a> TypeMapVisitor<'a> {
    pub fn new(
        function_types: &'a mut DenseHashMap<*mut AstExprFunction, String>,
        local_types: &'a mut DenseHashMap<*mut AstLocal, LuauBytecodeType>,
        expr_types: &'a mut DenseHashMap<*mut AstExpr, LuauBytecodeType>,
        host_vector_type: *const c_char,
        userdata_types: &'a DenseHashMap<AstName, u8>,
        builtin_types: &'a BuiltinAstTypes,
        builtin_calls: &'a DenseHashMap<*mut AstExprCall, i32>,
        globals: &'a DenseHashMap<AstName, Global>,
        library_member_type_cb: LibraryMemberTypeCallback,
        bytecode: &'a mut BytecodeBuilder,
    ) -> Self {
        Self {
            function_types,
            local_types,
            expr_types,
            host_vector_type,
            userdata_types,
            builtin_types,
            builtin_calls,
            globals,
            library_member_type_cb,
            bytecode,
            type_aliases: DenseHashMap::new(AstName::new()),
            type_alias_stack: Vec::new(),
            resolved_locals: DenseHashMap::new(core::ptr::null_mut()),
            resolved_exprs: DenseHashMap::new(core::ptr::null_mut()),
            function_return_types: DenseHashMap::new(core::ptr::null_mut()),
        }
    }
}
