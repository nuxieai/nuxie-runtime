//! Source: `Compiler/src/Types.cpp:253-951`

use crate::enums::global::Global;
use crate::records::builtin_ast_types::BuiltinAstTypes;
use crate::type_aliases::library_member_type_callback::LibraryMemberTypeCallback;
use alloc::string::String;
use alloc::vec::Vec;
use luaur_ast::records::ast_array::AstArray;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_stat_type_alias::AstStatTypeAlias;
use luaur_ast::records::ast_type::AstType;
use luaur_ast::records::ast_visitor::AstVisitor;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;
use luaur_common::records::dense_hash_map::DenseHashMap;

#[derive(Debug)]
pub struct TypeMapVisitor<'a> {
    pub(crate) function_types: &'a mut DenseHashMap<*mut AstExprFunction, String>,
    pub(crate) local_types: &'a mut DenseHashMap<*mut AstLocal, LuauBytecodeType>,
    pub(crate) expr_types: &'a mut DenseHashMap<*mut AstExpr, LuauBytecodeType>,
    pub(crate) host_vector_type: *const core::ffi::c_char,
    pub(crate) userdata_types: &'a DenseHashMap<AstName, u8>,
    pub(crate) builtin_types: &'a BuiltinAstTypes,
    pub(crate) builtin_calls: &'a DenseHashMap<*mut AstExprCall, i32>,
    pub(crate) globals: &'a DenseHashMap<AstName, Global>,
    pub(crate) library_member_type_cb: LibraryMemberTypeCallback,
    pub(crate) bytecode: &'a mut BytecodeBuilder,

    pub(crate) type_aliases: DenseHashMap<AstName, *mut AstStatTypeAlias>,
    pub(crate) type_alias_stack: Vec<(AstName, *mut AstStatTypeAlias)>,
    pub(crate) resolved_locals: DenseHashMap<*mut AstLocal, *const AstType>,
    pub(crate) resolved_exprs: DenseHashMap<*mut AstExpr, *const AstType>,
    pub(crate) function_return_types: DenseHashMap<*mut AstLocal, *const AstType>,
}

impl<'a> AstVisitor for TypeMapVisitor<'a> {
    fn visit_stat_block(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_stat_block(node as *mut luaur_ast::records::ast_stat_block::AstStatBlock)
    }

    fn visit_stat_repeat(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_stat_repeat(node as *mut luaur_ast::records::ast_stat_repeat::AstStatRepeat)
    }

    fn visit_stat_for(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_stat_for(node as *mut luaur_ast::records::ast_stat_for::AstStatFor)
    }

    fn visit_stat_for_in(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_stat_for_in(node as *mut luaur_ast::records::ast_stat_for_in::AstStatForIn)
    }

    fn visit_stat_local_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_stat_local_function(
            node as *mut luaur_ast::records::ast_stat_local_function::AstStatLocalFunction,
        )
    }

    fn visit_expr_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_function(
            node as *mut luaur_ast::records::ast_expr_function::AstExprFunction,
        )
    }

    fn visit_expr_local(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_local(node as *mut luaur_ast::records::ast_expr_local::AstExprLocal)
    }

    fn visit_stat_local(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_stat_local(node as *mut luaur_ast::records::ast_stat_local::AstStatLocal)
    }

    fn visit_expr_index_expr(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_index_expr(
            node as *mut luaur_ast::records::ast_expr_index_expr::AstExprIndexExpr,
        )
    }

    fn visit_expr_index_name(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_index_name(
            node as *mut luaur_ast::records::ast_expr_index_name::AstExprIndexName,
        )
    }

    fn visit_expr_unary(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_unary(node as *mut luaur_ast::records::ast_expr_unary::AstExprUnary)
    }

    fn visit_expr_binary(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_binary(node as *mut luaur_ast::records::ast_expr_binary::AstExprBinary)
    }

    fn visit_expr_group(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_group(node as *mut luaur_ast::records::ast_expr_group::AstExprGroup)
    }

    fn visit_expr_type_assertion(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_type_assertion(
            node as *mut luaur_ast::records::ast_expr_type_assertion::AstExprTypeAssertion,
        )
    }

    fn visit_expr_constant_bool(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_constant_bool(
            node as *mut luaur_ast::records::ast_expr_constant_bool::AstExprConstantBool,
        )
    }

    fn visit_expr_constant_number(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_constant_number(
            node as *mut luaur_ast::records::ast_expr_constant_number::AstExprConstantNumber,
        )
    }

    fn visit_expr_constant_integer(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_constant_integer(
            node as *mut luaur_ast::records::ast_expr_constant_integer::AstExprConstantInteger,
        )
    }

    fn visit_expr_constant_string(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_constant_string(
            node as *mut luaur_ast::records::ast_expr_constant_string::AstExprConstantString,
        )
    }

    fn visit_expr_interp_string(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_interp_string(
            node as *mut luaur_ast::records::ast_expr_interp_string::AstExprInterpString,
        )
    }

    fn visit_expr_if_else(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_if_else(
            node as *mut luaur_ast::records::ast_expr_if_else::AstExprIfElse,
        )
    }

    fn visit_expr_call(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_call(node as *mut luaur_ast::records::ast_expr_call::AstExprCall)
    }
}
