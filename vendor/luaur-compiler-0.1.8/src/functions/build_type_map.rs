use alloc::string::String;
use core::ffi::c_char;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_node::AstNode;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;
use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::enums::global::Global;
use crate::records::builtin_ast_types::BuiltinAstTypes;
use crate::records::type_map_visitor::TypeMapVisitor;
use crate::type_aliases::library_member_type_callback::LibraryMemberTypeCallback;

pub fn build_type_map(
    function_types: &mut DenseHashMap<*mut AstExprFunction, String>,
    local_types: &mut DenseHashMap<*mut AstLocal, LuauBytecodeType>,
    expr_types: &mut DenseHashMap<*mut AstExpr, LuauBytecodeType>,
    root: *mut AstNode,
    host_vector_type: *const c_char,
    userdata_types: &DenseHashMap<AstName, u8>,
    builtin_types: &BuiltinAstTypes,
    builtin_calls: &DenseHashMap<*mut AstExprCall, i32>,
    globals: &DenseHashMap<AstName, Global>,
    library_member_type_cb: LibraryMemberTypeCallback,
    bytecode: &mut BytecodeBuilder,
) {
    let mut visitor = TypeMapVisitor::new(
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
    );

    unsafe {
        luaur_ast::visit::ast_node_visit(root, &mut visitor);
    }
}
